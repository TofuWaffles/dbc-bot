use crate::bracket_tournament::config::{get_config, make_player_doc};
use crate::bracket_tournament::{api, region::Region};
use crate::database_utils::add::add_player;
use crate::database_utils::find::find_tag;
use crate::discord::prompt::prompt;
use crate::misc::{get_difficulty, CustomError};
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::serenity_prelude as serenity;
use poise::ReplyHandle;
use std::ops::Deref;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tracing::info;
const TIMEOUT: u64 = 120;
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
    display_register_region(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    info!("Registering menu is run");
    while let Some(mci) = &cic.next().await {
        info!("Inside the loop");
        match mci.data.custom_id.as_str() {
            "APAC" | "EU" | "NASA" => {
                register.region = Some(Region::find_key(mci.data.custom_id.as_str()).unwrap());
                mci.defer(&ctx.http()).await?;
                register_tag(ctx, msg).await?;
            }
            "open_modal" => {
                register.tag = Some(create_modal_tag(ctx, mci.clone()).await?);
                if account_available(ctx, register.tag.clone().unwrap()).await? {
                    register.player = display_confirmation(ctx, msg, &register).await?;
                } else {
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
                continue;
            }
        }
    }
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
            let club = match player["club"]["name"] {
                serde_json::Value::Null => "No Club",
                _ => player["club"]["name"].as_str().unwrap(),
            };
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
                            ("Club", club, true),
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
            prompt(
                ctx,
                msg,
                "Failed to find your account!",
                "Please try again with another account!",
                None,
                None,
            )
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
    let tag: String = match &register.tag {
        Some(tag) if tag.starts_with('#') => tag[1..].to_string(),
        Some(_) | None => "".to_string(),
    };
    msg
    .edit(*ctx, |s| {
        s.components(|c| c)
         .embed(|e| {
            e.title("**You have successfully registered!**")
                .description(format!("We have collected your information!\nYour player tag {} has been registered with the region {}\n You can safely dismiss this.", tag.to_uppercase(), register.region.clone().unwrap()))
        })
    }).await?;
    add_player(&ctx, &register.player, &register.region).await?;
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
    let config = get_config(ctx, &region.clone().unwrap()).await;
    let role = match config.get_str("role") {
        Ok(role) => Some(role),
        Err(_) => {
            msg.edit(*ctx, |s| {
                s.embed(|e| {
                    e.title("Failed to assign role!")
                        .description("Please contact Host or Moderators for this issue!")
                })
            })
            .await?;
            None
        }
    };
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
    match member
        .add_role((*ctx).http(), role.unwrap().parse::<u64>().unwrap())
        .await
    {
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

async fn account_available(ctx: &Context<'_>, tag: String) -> Result<bool, Error> {
    match find_tag(ctx, tag.as_str()).await {
        Some(_) => Ok(false),
        None => Ok(true),
    }
}

async fn already_used(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.embed(|e|{
            e.title("This account has already been used!")
            .description("This account has already been used to register by another player! Please try again with another account!")
        })
    })
    .await?;
    Ok(())
}
