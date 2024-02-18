use crate::database::config::get_config;
use crate::database::find::{find_player_by_discord_id, find_round_from_config};
use crate::database::remove::remove_player;
use crate::discord::prompt::prompt;
use crate::discord::role::get_region_role_id;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::Document;
use poise::ReplyHandle;
use tracing::error;
use std::sync::Arc;
const TIMEOUT: u64 = 120;

struct PlayerDisqualification {
    user_id: Option<String>,
    region: Region,
}

#[derive(Debug, poise::Modal)]
#[name = "Disqualify Modal"]
struct DisqualifyModal {
    #[name = "Disqualify Player whose ID is:"]
    #[placeholder = "Make sure the user ID is provided, not the username"]
    user_id: String,
}

pub async fn disqualify_players(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.ephemeral(true)
            .reply(true)
            .content("Attempting to disqualify player...")
    })
    .await?;
    let mut disqualification = PlayerDisqualification {
        user_id: None,
        region: region.clone(),
    };
    disqualify_id(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "open_modal" => {
                disqualification.user_id = Some(create_disqualify_modal(ctx, mci.clone()).await?);
                match find_player_by_discord_id(
                    ctx,
                    &(disqualification.region.clone()),
                    disqualification
                        .user_id
                        .clone()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                        find_round_from_config(&get_config(ctx, region).await)
                ).await
                {
                    Ok(Some(player)) => display_confirmation(ctx, msg, &player).await?,
                    Ok(None) => {
                        msg.edit(*ctx, |s| {
                            s.reply(true)
                                .embed(|e| 
                                    e.title("No player found")
                                    .description("No player is found for this ID"))
                                .components(|c|c)
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
                    &disqualification.region.clone(),
                    disqualification
                        .user_id
                        .clone()
                        .unwrap()
                        .parse::<u64>()
                        .unwrap(),
                        find_round_from_config(&get_config(ctx, region).await)

                ).await {
                    Ok(Some(player)) => {
                        if let Ok(round) = remove_player(ctx, &player, region).await {
                            msg.edit(*ctx,|s| {
                            s.reply(true)
                                .embed(|e| {
                                    e.description(format!(
                                        "Successfully disqualified player: {}({}) with respective Discord <@{}> at round {round}",
                                        player.get_str("name").unwrap(),
                                        player.get_str("tag").unwrap(),
                                        &disqualification.user_id.clone().unwrap(),
                                    ))
                                })
                        })
                        .await?;
                    match ctx.guild().unwrap().member(ctx.http(), disqualification.user_id.clone().unwrap().parse::<u64>().unwrap()).await{
                        Ok(mut member) => {
                            match member.remove_role(ctx.http(), get_region_role_id(ctx, region).await.unwrap()).await{
                                Ok(_) => {
                                    msg.edit(*ctx, |s| {
                                        s.embed(|e| {
                                            e.description("Successfully removed the role from the user")
                                        })
                                    }).await?;
                                },
                                Err(e) => {
                                    error!("{e}");
                                    msg.edit(*ctx, |s| {
                                        s.embed(|e| {
                                            e.description("Failed to remove the role from the user")
                                        })
                                    }).await?;
                                }
                            }
                        }
                        Err(e) => {
                            error!("{e}");
                            return Ok(())
                        },
                    };
                }
            }
                    Ok(None) => {}
                    Err(e) => {
                        error!("{e}");
                    }
                   
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


async fn disqualify_id(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("🔨 Disqualify Players - Step 1: Enter the user ID")
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
                .title("🔨 Disqualify Players - Step 2: User confirmation")
                .description(
                    "**Please confirm this is the player that you would like to disqualify.**",
                )
                .fields(vec![
                    ("Mention",format!("<@{}>", player.get_str("discord_id").unwrap()),true),
                    ("Region", player.get_str("region").unwrap().to_string(), true),
                    ("Name", player.get_str("name").unwrap().to_string(), true),
                    ("Tag", player.get_str("tag").unwrap().to_string(), true),
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
