use crate::brawlstars::api::{request, APIResult};
use crate::brawlstars::player::stat;
use crate::database::add::add_player;
use crate::database::config::{get_config, make_player_doc};
use crate::database::find::find_tag;
use crate::database::open::registration_region_open;
use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::{CustomError, Region};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::serenity_prelude::{self as serenity, ReactionType};
use poise::ReplyHandle;
use std::ops::Deref;
use std::sync::Arc;
use strum::IntoEnumIterator;
use tracing::{error, info};

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
    #[placeholder = "The tag should start with # For instance, #ABC123"]
    #[min_length = 5]
    #[max_length = 10]
    tag: String,
}
pub async fn register_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
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
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "APAC" | "EU" | "NASA" => {
                register.region = Some(Region::find_key(mci.data.custom_id.as_str()).unwrap());
                mci.defer(&ctx.http()).await?;
                if registration_region_open(ctx, &register.region.clone().unwrap()).await {
                    register_tag(ctx, msg).await?;
                    continue;
                } else {
                    prompt(
                        ctx,
                        msg,
                        "Registration is not open for this region!",
                        "Please try again later!",
                        None,
                        Some(0xFF0000),
                    )
                    .await?;
                    return Ok(());
                }
            }
            "open_modal" => {
                register.tag = Some(create_modal_tag(ctx, mci.clone()).await?.to_uppercase());
                match find_tag(ctx, &register.tag.clone().unwrap()).await {
                    Some(player) => {
                        return already_used(ctx, msg, player).await;
                    }
                    None => {
                        register.player = display_confirmation(ctx, msg, &register).await?;
                        continue;
                    }
                }
            }
            "confirm" => return confirm(ctx, msg, &register).await,
            "cancel" => {
                mci.defer(&ctx.http()).await?;
                return cancel(ctx, msg).await;
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
    msg.edit(*ctx, |b| {
        b.embed(|e| {
            e.title("Step 1: Select your region").description(
                r#"
The tournament is available for all 3 regions: 
-🌎: North America & South America.
-🌍: Europe.
-🌏: Asia & Oceania."#,
            )
        })
        .components(|c| {
            c.create_action_row(|a| {
                for region in Region::iter() {
                    a.create_button(|b| {
                        b.custom_id(region.short())
                            .emoji(ReactionType::Unicode(region.get_emoji()))
                    });
                }
                a
            })
        })
    })
    .await?;
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
    match request("player", register.tag.clone().unwrap().as_str()).await {
        Ok(APIResult::Successful(player)) => {
            msg.edit(*ctx, |s| {
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
            })
            .await?;
            stat(ctx, msg, &player, &register.region.clone().unwrap(), None).await?;

            Ok(Some(make_player_doc(
                &player,
                &ctx.author_member().await.unwrap().user.id.to_string(),
                &register.region.clone().unwrap(),
            )))
        }
        Ok(APIResult::APIError(_)) => {
            prompt(
                ctx,
                msg,
                "The API is so uncanny!",
                "Please try again later",
                None,
                Some(0xFF0000),
            )
            .await?;
            Ok(None)
        }
        Ok(APIResult::NotFound(_)) => {
            prompt(
                ctx,
                msg,
                "Failed to find your account!",
                "We failed to find your account! Please try again!",
                None,
                Some(0xFF0000),
            )
            .await?;
            Ok(None)
        }
        Err(e) => {
            info!(e);
            prompt(
                ctx,
                msg,
                "Something went wrong!",
                "Please try again later!",
                None,
                Some(0xFF0000),
            )
            .await?;
            Ok(None)
        }
    }
}

async fn confirm(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    register: &PlayerRegistration,
) -> Result<(), Error> {
    add_player(
        ctx,
        register.player.clone().unwrap(),
        &register.region.clone().unwrap(),
    )
    .await?;
    assign_role(ctx, msg, &register.region).await?;
    prompt(
        ctx,
        msg,
        "Congratulations! You are one of our participants!",
        format!("<@{}>, we have collected your registration with the account tagged {}\nYou can run </index:1181542953542488205> again to view your registration!", ctx.author().id, register.player.clone().unwrap().get_str("tag").unwrap()),
        None,
        Some(0xFFFF00)).await
}

async fn cancel(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "You have cancelled your registration!",
        "You can always register again by running </index:1181542953542488205>",
        None,
        Some(0xFF0000),
    )
    .await
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
            prompt(
                ctx,
                msg,
                "Failed! Unable to find role!",
                "Please contact Host or Moderator for this issue",
                None,
                Some(0xFF0000),
            )
            .await?;
            error!(
                "Failed to get role for region {:?}",
                region.clone().unwrap()
            );
            return Err(Box::new(CustomError(format!(
                "Failed to get role for region {:?}",
                region.clone().unwrap()
            ))));
        }
    };
    let mut member = match ctx.author_member().await {
        Some(m) => m.deref().to_owned(),
        None => {
            let user = *ctx.author().id.as_u64();
            prompt(
                ctx,
                msg,
                "Failed! Unable to assign role!",
                "Please contact Host or Moderator for this issue",
                None,
                Some(0xFF0000),
            )
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
            prompt(
                ctx,
                msg,
                "Failed! Unable to assign role!",
                "Please contact Host or Moderator for this issue",
                None,
                Some(0xFF0000),
            )
            .await?;
            info!("Failed to assign role for <@{}>", user);
            Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{}>",
                user
            ))))
        }
    }
}

async fn already_used(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    let discord_id = player.get_str("discord_id").unwrap();
    prompt(
        ctx,
        msg,
        "This account has already been used!",
        format!("This account has already been registered by <@{}>. If this is unwanted, please issue to the Host or Moderator team!", discord_id),
        None,
        Some(0xFF0000)
    ).await?;
    Ok(())
}
