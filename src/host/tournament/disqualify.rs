use crate::database::find::find_player_by_discord_id;
use crate::database::remove::remove_player;
use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::Document;
use poise::ReplyHandle;
use std::sync::Arc;
use strum::IntoEnumIterator;
const TIMEOUT: u64 = 120;

struct PlayerDisqualification {
    user_id: Option<String>,
    region: Option<Region>,
}

#[derive(Debug, poise::Modal)]
#[name = "Disqualify Modal"]
struct DisqualifyModal {
    #[name = "Enter the user ID of the player you want to disqualify"]
    #[placeholder = "Make sure the user ID is provided, not the username"]
    user_id: String,
}

pub async fn disqualify_players(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.ephemeral(true)
            .reply(true)
            .content("Attempting to disqualify player...")
    })
    .await?;
    let mut disqualification = PlayerDisqualification {
        user_id: None,
        region: None,
    };
    disqualify_region(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "APAC" | "EU" | "NASA" => {
                disqualification.region =
                    Some(Region::find_key(mci.data.custom_id.as_str()).unwrap());
                mci.defer(&ctx.http()).await?;
                disqualify_id(ctx, msg).await?;
                continue;
            }
            "open_modal" => {
                disqualification.user_id = Some(create_disqualify_modal(ctx, mci.clone()).await?);
                match find_player_by_discord_id(
                    ctx,
                    &(disqualification.region.clone().unwrap()),
                    disqualification
                        .user_id
                        .clone()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                )
                .await
                {
                    Ok(Some(player)) => display_confirmation(ctx, msg, &player).await?,
                    Ok(None) => {
                        msg.edit(*ctx, |s| {
                            s.reply(true)
                                .ephemeral(true)
                                .embed(|e| e.description("No player is found for this ID"))
                        })
                        .await?;
                        return Ok(());
                    }
                    Err(_) => {}
                }
            }
            "confirm" => {
                match find_player_by_discord_id(
                    ctx,
                    &disqualification.region.clone().unwrap(),
                    disqualification
                        .user_id
                        .clone()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                )
                .await
                {
                    Ok(Some(player)) => {
                        if let Ok(round) = remove_player(ctx, &player).await {
                            ctx.send(|s| {
                            s.reply(true)
                                .ephemeral(true)
                                .embed(|e| {
                                    e.description(format!(
                                        "Successfully disqualified player: {}({}) with respective Discord <@{}> at round {}",
                                        player.get("name").unwrap().as_str().unwrap(),
                                        player.get("tag").unwrap().as_str().unwrap(),
                                        &disqualification.user_id.unwrap().to_string(),
                                        round
                                    ))
                                })
                        })
                        .await?;
                            return Ok(());
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
            "cancel" => {
                mci.defer(&ctx.http()).await?;
                prompt(
                    ctx,
                    msg,
                    "Player disqualification cancelled",
                    "You can return to this menu by running </index:1181542953542488205>",
                    None,
                    None,
                )
                .await?;
            }
            _ => {
                continue;
            }
        }
    }
    Ok(())
}

async fn disqualify_region(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("🔨 Disqualify Players - Step 1: Select their region")
            .description("Select the region of the player you would like to disqualify from the tournament.")
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

async fn disqualify_id(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("🔨 Disqualify Players - Step 2: Enter the user ID")
            .description("Please enter the user ID of the player you want to disqualify. See [this guide](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-) for more information.")
        })
        .components(|c|{
            c.create_action_row(|a|{
                a.create_button(|b|{
                    b.custom_id("open_modal")
                    .label("Disqualify Player")
                })
        })
    })}).await?;
    Ok(())
}

async fn display_confirmation(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: &Document,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Confirm")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .custom_id("confirm")
                })
                .create_button(|b| {
                    b.label("Cancel")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .custom_id("cancel")
                })
            })
        })
        .reply(true)
        .ephemeral(true)
        .embed(|e| {
            e.author(|a| a.name(ctx.author().name.clone()))
                .title("🔨 Disqualify Players - Step 3: User confirmation")
                .description(
                    "**Please confirm this is the player that you would like to disqualify.**",
                )
                .fields(vec![
                    (
                        "Mention",
                        format!("<@{}>", player.get("discord_id").unwrap().as_str().unwrap()),
                        true,
                    ),
                    ("Region", player.get("region").unwrap().to_string(), true),
                    ("Name", player.get("name").unwrap().to_string(), true),
                    ("Tag", player.get("tag").unwrap().to_string(), true),
                ])
                .timestamp(ctx.created_at())
        })
    })
    .await?;

    Ok(())
}

pub async fn create_disqualify_modal(
    ctx: &Context<'_>,
    mci: Arc<poise::serenity_prelude::MessageComponentInteraction>,
) -> Result<String, Error> {
    loop {
        let result = poise::execute_modal_on_component_interaction::<DisqualifyModal>(
            ctx,
            mci.clone(),
            None,
            None,
        )
        .await?;
        match result {
            Some(data) => {
                return Ok(data.user_id);
            }
            None => continue,
        }
    }
}
