use crate::bracket_tournament::{api, models};
use crate::misc::{get_difficulty, is_in_db, QuoteStripper};
use crate::{Context, Error};
use mongodb::bson::doc;
use poise::serenity_prelude::{self as serenity};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, poise::ChoiceParameter)]
pub enum Region {
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

/// Sign up for Discord Brawl Cup Tournament!
#[poise::command(slash_command, guild_only, track_edits)]
pub async fn register(
    ctx: Context<'_>,
    #[description = "Put your player tag here (without #)"] tag: String,
    #[description = "Put your region here"] region: models::Region,
) -> Result<(), Error> {
    // ctx.defer_ephemeral().await?;
    ctx.defer().await?;
    //Check whether a player has registered or not by their Discord id
    match is_in_db(&ctx).await {
        Some(_) => {
            ctx.send(|s|{
            s.reply(true)
            .ephemeral(true)
            .embed(|e|{
                e.title("**You have already registered!**")
                .description("You have already registered for the tournament!
                    \n If you want to participate the event with a different account, please </derigster:1146092020843155496> first and run this again!")
            })
        }).await?;
            return Ok(());
        }
        None => {}
    }
    let registry_confirm: u64 = format!("{}1", ctx.id()).parse().unwrap(); //Message ID concatenates with 1 which indicates true
    let registry_cancel: u64 = format!("{}0", ctx.id()).parse().unwrap(); //Message ID concatenates with 0 which indicates false
    let endpoint = api::get_api_link("player", &tag.to_uppercase());

    match api::request(&endpoint).await {
        Ok(player) => {
            // let embed = player_embed(&player, &ctx, &region);
            ctx.send(|s| {
                s.components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.label("Confirm")
                                    .style(poise::serenity_prelude::ButtonStyle::Success)
                                    .custom_id(registry_confirm)
                            })
                            .create_button(|b| {
                                b.label("Cancel")
                                    .style(poise::serenity_prelude::ButtonStyle::Danger)
                                    .custom_id(registry_cancel)
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

            while let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(std::time::Duration::from_secs(120))
                .await
            {
                if mci.data.custom_id == registry_confirm.to_string(){
                    let mut confirm_prompt = mci.message.clone();
                    confirm_prompt
                        .edit(ctx, |s| {
                            s.components(|c| c)
                             .embed(|e| {
                                e.title("**You have successfully registered!**")
                                    .description(format!("We have collected your information!\nYour player tag #{} has been registered with the region {}", tag.to_uppercase(), region))
                            })
                        }).await?;

                    let data = serenity::json::json!({
                        "name": player["name"].to_string().strip_quote(),
                        "tag": player["tag"].to_string().strip_quote(),
                        "id": ctx.author_member().await.unwrap().user.id.to_string(),
                        "region": format!("{:?}", region),
                    });

                    let collection = ctx.data().database.collection("PlayerDB");
                    println!(
                        "Player {}({}) has been registered!",
                        player["name"].to_string().strip_quote(),
                        player["tag"].to_string().strip_quote()
                    );

                    match collection.insert_one(data, None).await {
                        Ok(_) => {}
                        Err(err) => match err.kind.as_ref() {
                            mongodb::error::ErrorKind::Command(code) => {
                                eprintln!("Command error: {:?}", code);
                            }
                            mongodb::error::ErrorKind::Write(code) => {
                                eprintln!("Write error: {:?}", code);
                            }
                            _ => {
                                eprintln!("Error: {:?}", err);
                            }
                        },
                    };
                } else {
                    let mut cancel_prompt = mci.message.clone();
                    cancel_prompt
                    .edit(ctx, |s| {
                        s.components(|c|{c})
                        .embed(|e| {
                            e.title("**Please try again**")
                                .description("You have cancelled your registration for the tournament! Please try again!")
                        })
                    })
                    .await?;
                }
                mci.create_interaction_response(ctx, |ir| {
                    ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
                })
                .await?;
                std::thread::sleep(std::time::Duration::from_secs(10));
                mci.message.delete(ctx).await?;

            }
        }
        Err(_) => {
            ctx.send(|s| {
                s.content("".to_string())
                    .reply(true)
                    .ephemeral(false)
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
