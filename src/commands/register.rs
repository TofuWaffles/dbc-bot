use crate::bracket_tournament::config::{get_config, make_player_doc};
use crate::bracket_tournament::{api, region::Region};
use crate::database_utils::find_discord_id::find_discord_id;
use crate::database_utils::find_tag::find_tag;
use crate::misc::{get_difficulty, QuoteStripper};
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::Document;
use mongodb::bson::doc;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::json::Value;
use poise::ReplyHandle;
use tracing::{error, info, instrument};

/// Sign up for Discord Brawl Cup Tournament!
#[instrument]
#[poise::command(slash_command, guild_only, track_edits)]
pub async fn register(
    ctx: Context<'_>,
    #[description = "Put your player tag here (without #)"] tag: String,
    #[description = "Put your region here"] region: Region,
) -> Result<(), Error> {
    info!("Attempting to register {}", ctx.author().tag());
    //Check whether registation has already closed
    ctx.defer_ephemeral().await?;
    let msg: ReplyHandle<'_> = ctx.say("Checking registration status...").await?;
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;

    if !register_opened(&ctx, &msg, &config).await? || !account_available(&ctx, &msg, &tag).await? {
        return Ok(());
    }
    if player_registered(&ctx, Some(region.clone()))
        .await?
        .is_some()
    {
        msg.edit(ctx,|s|{
            s.embed(|e|{
                e.title("**You have already registered!**")
                .description("You have already registered for the tournament! If you want to deregister, please use the </deregister:1146092020843155496> command!")
            })
        }).await?;
    }

    let endpoint = api::get_api_link("player", &tag.to_uppercase());

    match api::request(&endpoint).await {
        Ok(player) => {
            // let embed = player_embed(&player, &ctx, &region);
            msg.edit(ctx,|s| {
                s.components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.label("Confirm")
                                    .style(poise::serenity_prelude::ButtonStyle::Success)
                                    .custom_id("Confirm")
                            })
                            .create_button(|b| {
                                b.label("Cancel")
                                    .style(poise::serenity_prelude::ButtonStyle::Danger)
                                    .custom_id("Cancel")
                            })
                        })
                    })
                    .reply(true)
                    .ephemeral(true)
                    .embed(|e| {
                        e.author(|a| a.name(ctx.author().name.clone()))
                        .title(format!("**{} ({})**", player["name"].to_string().strip_quote(), player["tag"].to_string().strip_quote()))
                        .description("**Please confirm this is the correct account that you are going to use during our tournament!**")
                        .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player["icon"]["id"]))
                        .fields(vec![
                            ("**Region**", region.to_string(), true),
                            ("Trophies", player["trophies"].to_string(), true),
                            ("Highest Trophies", player["highestTrophies"].to_string(), true),
                            ("3v3 Victories", player["3vs3Victories"].to_string(), true),
                            ("Solo Victories", player["soloVictories"].to_string(), true),
                            ("Duo Victories", player["duoVictories"].to_string(), true),
                            ("Best Robo Rumble Time", get_difficulty(&player["bestRoboRumbleTime"]),true),
                            ("Club", player["club"]["name"].to_string().strip_quote(), true),
                        ])
                        .timestamp(ctx.created_at())
                    })
            }
          )
            .await?;
            let resp = msg.clone().into_message().await?;
            let cib = resp
                .await_component_interactions(&ctx.serenity_context().shard)
                .timeout(std::time::Duration::from_secs(120));
            let mut cic = cib.build();

            while let Some(mci) = &cic.next().await {
                match mci.data.custom_id.as_str() {
                    "Confirm" => {
                        msg
                            .edit(ctx, |s| {
                                s.components(|c| c)
                                 .embed(|e| {
                                    e.title("**You have successfully registered!**")
                                        .description(format!("We have collected your information!\nYour player tag #{} has been registered with the region {}", tag.to_uppercase(), region))
                                })
                            }).await?;

                        insert_player(&player, &ctx, region.clone()).await?;
                    }
                    "Cancel" => {
                        msg
                    .edit(ctx, |s| {
                        s.components(|c|{c})
                        .embed(|e| {
                            e.title("**Please try again**")
                                .description("You have cancelled your registration for the tournament! Please try again!")
                        })
                    })
                    .await?;
                    }
                    _ => {
                        unreachable!("Cannot get here..");
                    }
                }
            }
        }
        Err(_) => {
            msg.edit(ctx, |s| {
                s.content("".to_string())
                    .reply(true)
                    .ephemeral(true)
                    .embed(|e| {
                        e.title("**We have tried very hard to find but...**")
                            .description(format!(
                                "No player is associated with the tag {}",
                                tag.to_uppercase()
                            ))
                            .field("Please try again!".to_string(), "".to_string(), true)
                    })
            })
            .await?;
        }
    }
    Ok(())
}

/// Remove your registration from Discord Brawl Cup.
#[instrument]
#[poise::command(slash_command, guild_only)]
pub async fn deregister(ctx: Context<'_>) -> Result<(), Error> {
    info!("Attempted to deregister user {}", ctx.author().tag());
    ctx.defer_ephemeral().await?;
    let msg: ReplyHandle<'_> = ctx.say("Checking registration status...").await?;
    let player = match player_registered(&ctx, None).await? {
        None => {
            msg.edit(ctx,|s|{
                s.reply(true)
                .embed(|e|{
                    e.title("**You have not registered!**")
                    .description("You have not registered for the tournament! If you want to register, please use the </register:1145363516325376031> command!")
                })
            }).await?;
            return Ok(());
        }
        Some(data) => data,
    };

    let region = Region::find_key(player.get("region").unwrap().as_str().unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;

    if !register_opened(&ctx, &msg, &config).await? {
        return Ok(());
    }

    msg.edit(ctx,|s|{
        s.components(|c|{
          c.create_action_row(|a|{
            a.create_button(|b|{
              b.label("Confirm")
              .style(serenity::ButtonStyle::Success)
              .custom_id("Confirm")
            })
            .create_button(|b|{
              b.label("Cancel")
              .style(serenity::ButtonStyle::Danger)
              .custom_id("Cancel")
            })
          })
        })
          .reply(true)
          .embed(|e|{
            e.title("**Are you sure you want to deregister?**")
            .description(format!("You are about to deregister from the tournament. Below information are what you told us!\nYour account name: **{}**\nWith your respective tag: **{}**\nAnd you are in the following region: **{}**", 
                                player.get("name").unwrap().to_string().strip_quote(), 
                                player.get("tag").unwrap().to_string().strip_quote(), 
                                Region::find_key(player.get("region").unwrap().to_string().strip_quote().as_str()).unwrap()) 
                        )
        })
    }).await?;

    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();

    while let Some(mci) = cic.next().await {
        match mci.data.custom_id.as_str() {
            "Confirm" => {
                let region =
                    Region::find_key(&player.get("region").unwrap().to_string().strip_quote());
                let player_data = ctx
                    .data()
                    .database
                    .regional_databases
                    .get(&region.unwrap())
                    .unwrap()
                    .collection::<Document>("Players");
                player_data
                    .delete_one(doc! {"_id": player.get("_id")}, None)
                    .await?;
                msg.edit(ctx,|s| {
                    s.components(|c| {c})
                        .embed(|e| {
                            e.title("**Deregistration is successful**")
                                .description(
                                "Seriously, are you leaving us? We hope to see you in the next tournament!",
                            )
                      })
                })
                .await?;
            }
            "Cancel" => {
                msg.edit(ctx, |s| {
                    s.components(|c| c).embed(|e| {
                        e.title("**Deregistration cancelled!**")
                            .description("Thanks for staying in the tournament with us!")
                    })
                })
                .await?;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }
    Ok(())
}

pub async fn register_opened(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    config: &Document,
) -> Result<bool, Error> {
    if !(config.get("registration").unwrap()).as_bool().unwrap() {
        msg.edit(*ctx, |s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("**Registration has already closed!**")
                    .description("Sorry, registration has already closed!")
            })
        })
        .await?;
        Ok(false)
    } else {
        Ok(true)
    }
}

pub async fn player_registered(
    ctx: &Context<'_>,
    region: Option<Region>,
) -> Result<Option<Document>, Error> {
    match find_discord_id(ctx, None, region).await {
        None => Ok(None),
        Some(player) => Ok(Some(player)),
    }
}

async fn account_available(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    tag: &str,
) -> Result<bool, Error> {
    if let Some(someone) = find_tag(ctx, &(tag.to_uppercase())).await {
        msg.edit(*ctx, |s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("**This account has been registered by some player already!**")
                    .description(format!(
                        "**{}** ({}) has already been registered with <@{}>.",
                        someone.get("name").unwrap().to_string().strip_quote(),
                        someone.get("tag").unwrap().to_string().strip_quote(),
                        someone.get("discord_id").unwrap().to_string().strip_quote()
                    ))
            })
        })
        .await?;
        Ok(false)
    } else {
        Ok(true)
    }
}

async fn insert_player(player: &Value, ctx: &Context<'_>, region: Region) -> Result<(), Error> {
    let data = make_player_doc(player,  &ctx.author_member().await.unwrap().user.id.to_string(), &region);
    let collection = ctx.data().database.regional_databases[&region].collection("Players");
    match collection.insert_one(data, None).await {
        Ok(_) => {}
        Err(err) => match err.kind.as_ref() {
            mongodb::error::ErrorKind::Command(code) => {
                error!("Command error: {:?}", code);
            }
            mongodb::error::ErrorKind::Write(code) => {
                error!("Write error: {:?}", code);
            }
            _ => {
                error!("Error: {:?}", err);
            }
        },
    };
    Ok(())
}
