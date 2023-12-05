use crate::bracket_tournament::config::{get_config, make_player_doc};
use crate::bracket_tournament::{api, region::Region};
use crate::database_utils::find_tag::find_tag;
use crate::misc::{get_difficulty, CustomError};
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::Document;
use mongodb::Collection;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::ButtonStyle;
use poise::ReplyHandle;
use std::ops::Deref;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tracing::{error, info};
const REGISTRATION_TIME: u64 = 120;
struct PlayerRegistration {
    tag: Option<String>,
    region: Option<Region>,
    player: Option<Document>,
}

#[derive(Debug, poise::Modal)]
#[name = "Player Tag"]
struct TagModal {
    #[name = "Enter your player tag:"]
    #[placeholder = "The tag should start with #. For instance, #ABC123"]
    #[min_length = 5]
    #[max_length = 10]
    tag: String,
}
pub async fn register_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    info!("Registering menu is run");

    let mut register = PlayerRegistration {
        tag: None,
        region: None,
        player: None,
    };
    display_register_menu(ctx, msg).await?; //Step 0
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(REGISTRATION_TIME));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "register" => {
                mci.defer(&ctx.http()).await?;
                display_register_region(ctx, msg).await?;
            }
            "APAC" | "EU" | "NASA" => {
                register.region = Some(Region::find_key(mci.data.custom_id.as_str()).unwrap());
                mci.defer(&ctx.http()).await?;
                register_tag(ctx, msg).await?;
            }
            "open_modal" => {
                register.tag = Some(create_modal_tag(ctx, mci.clone()).await?);
                if account_available(ctx, register.tag.clone().unwrap()).await?{
                    register.player = display_confirmation(ctx, msg, &register).await?;
                } else{
                    already_used(ctx, msg).await?;
                    break;
                }
            }
            "confirm" => {
                confirm(ctx, msg, &register).await?;
                break;
            }
            "cancel" => {
                cancel(ctx, msg).await?;
                break;
            }
            _ => {
                unreachable!("Cannot get here..");
            }
        }
    }
    Ok(())
}

//Step 0
async fn display_register_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b| {
        b.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Register")
                        .style(ButtonStyle::Primary)
                        .custom_id("register")
                })
            })
        })
        .embed(|e| {
            e.title("Registration")
                .description("Press the button below to start registration")
        })
    })
    .await?;
    Ok(())
}

//Step 1
async fn display_register_region(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("Step 1: Select your region")
            .description("- The tournament is available for all 3 regions:\n - Asia & Oceania\n - Europe\n - North America & South America")
        })
        .components(|c|{
            c.create_action_row(|a|{
                for region in Region::iter(){
                    a.create_button(|b|{
                        b.custom_id(format!("{:?}", region))
                        .label(format!("{}", region))
                    });
                };
                a
            })
        })
    }).await?;
    Ok(())
}

//Step 2
async fn register_tag(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("Step 2: Enter your player tag!")
            .description("Please enter your player tag. You can find them in your game with the instruction below.")
            .image("https://cdn.brawlstats.com/creatives/2021-profile-hints.png")
        })
        .components(|c|{
            c.create_action_row(|a|{
                a.create_button(|b|{
                    b.custom_id("open_modal")
                    .label("Click/tap here to enter your player tag!")
                })
        })
    })}).await?;
    Ok(())
}

async fn create_modal_tag(
    ctx: &Context<'_>,
    mci: Arc<serenity::MessageComponentInteraction>,
) -> Result<String, Error> {
    loop {
        let result =
            poise::execute_modal_on_component_interaction::<TagModal>(ctx, mci.clone(), None, None)
                .await?;
        match result {
            Some(data) => {
                return Ok(data.tag);
            }
            None => continue,
        }
    }
}
//Step 3
async fn display_confirmation(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    register: &PlayerRegistration,
) -> Result<Option<Document>, Error> {
    match api::request("player", register.tag.clone().unwrap().as_str()).await {
        Ok(player) => {
            msg.edit(*ctx,|s| {
                s.components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.label("Confirm")
                                    .style(poise::serenity_prelude::ButtonStyle::Success)
                                    .custom_id("confirm")
                            })
                            .create_button(|b| {
                                b.label("Cancel")
                                    .style(poise::serenity_prelude::ButtonStyle::Danger)
                                    .custom_id("cancel")
                            })
                        })
                    })
                    .reply(true)
                    .ephemeral(true)
                    .embed(|e| {
                        e.author(|a| a.name(ctx.author().name.clone()))
                        .title(format!("Step 3: Verify the following account: **{} ({})**", player["name"].as_str().unwrap(), player["tag"].as_str().unwrap()))
                        .description("**Please confirm this is the correct account that you are going to use during our tournament!**")
                        .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player["icon"]["id"]))
                        .fields(vec![
                            ("**Region**", format!("{}",register.region.clone().unwrap()).as_str(), true),
                            ("Trophies", player["trophies"].to_string().as_str(), true),
                            ("Highest Trophies", player["highestTrophies"].to_string().as_str(), true),
                            ("3v3 Victories", player["3vs3Victories"].to_string().as_str(), true),
                            ("Solo Victories", player["soloVictories"].to_string().as_str(), true),
                            ("Duo Victories", player["duoVictories"].to_string().as_str(), true),
                            ("Best Robo Rumble Time", &get_difficulty(&player["bestRoboRumbleTime"]),true),
                            ("Club", player["club"]["name"].as_str().unwrap(), true),
                        ])
                        .timestamp(ctx.created_at())
                    })
            }
          )
            .await?;
            return Ok(Some(make_player_doc(
                &player,
                &ctx.author_member().await.unwrap().user.id.to_string(),
                &register.region.clone().unwrap(),
            )));
        }
        Err(_) => {
            msg.edit(*ctx, |s| {
                s.content("".to_string())
                    .reply(true)
                    .components(|c|c)
                    .embed(|e|{
                        e.title("**We have tried very hard to find but...**")
                            .description(format!(
                                "No player is associated with the tag {}",
                                register.tag.clone().unwrap().to_uppercase()
                            ))
                            .field("Please try again!".to_string(), "".to_string(), true)
                    })
            })
            .await?;
            return Ok(None);
        }
    }
}

async fn confirm(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    register: &PlayerRegistration,
) -> Result<(), Error> {
    msg
    .edit(*ctx, |s| {
        s.components(|c| c)
         .embed(|e| {
            e.title("**You have successfully registered!**")
                .description(format!("We have collected your information!\nYour player tag #{} has been registered with the region {}\n You can safely dismiss this.", register.tag.clone().unwrap().to_uppercase(), register.region.clone().unwrap()))
        })
    }).await?;
    insert_player(&ctx, &register.player, &register.region).await?;
    assign_role(&ctx, &msg, &register.region).await?;
    Ok(())
}

async fn cancel(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.components(|c| c).embed(|e| {
            e.title("**Please try again**").description(
                "You have cancelled your registration for the tournament! Please try again!",
            )
        })
    })
    .await?;
    Ok(())
}

async fn assign_role(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Option<Region>,
) -> Result<(), Error> {
    let database = ctx
        .data()
        .database
        .regional_databases
        .get(&region.clone().unwrap())
        .unwrap();
    let config = get_config(database).await;
    let role_id = config
        .get("role")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let mut member = match ctx.author_member().await {
        Some(m) => m.deref().to_owned(),
        None => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Assigning role failed! Please contact Moderators for this issue")
            })
            .await?;
            info!("Failed to assign role for <@{}>", user);
            return Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{}>",
                user
            ))));
        }
    };
    match member.add_role((*ctx).http(), role_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Assigning role failed! Please contact Moderators for this issue")
            })
            .await?;
            info!("Failed to assign role for <@{}>", user);
            Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{}>",
                user
            ))))
        }
    }
}

async fn insert_player(
    ctx: &Context<'_>,
    player: &Option<Document>,
    region: &Option<Region>,
) -> Result<(), Error> {
    let collection: Collection<Document> =
        ctx.data().database.regional_databases[&region.clone().unwrap()].collection("Players");
    match collection.insert_one(player.clone().unwrap(), None).await {
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

async fn account_available(ctx: &Context<'_>, tag: String) -> Result<bool, Error>{
    match find_tag(ctx, tag.as_str()).await{
        Some(_) => Ok(false),
        None => Ok(true)
    }
}

async fn already_used(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error>{
    msg.edit(*ctx, |s| {
        s.embed(|e|{
            e.title("This account has already been used!")
            .description("This account has already been used to register by another player! Please try again with another account!")
        })
    })
    .await?;
    Ok(())
}