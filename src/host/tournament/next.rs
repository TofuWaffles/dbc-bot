use crate::database::find::find_all_false_battles;
use crate::database::update::update_round_config;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::stream::StreamExt;
use mongodb::bson::{self, Document};
use poise::ReplyHandle;
use std::collections::HashMap;
use tracing::{error, info};

use super::download::{compact, get_downloadable_ids};
const TIMEOUT: u64 = 300;
pub async fn display_next_round(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    info!("Next round is triggered by {}", ctx.author().name);
    let battles = display_false_battles(ctx, region).await;
    if battles.is_empty() {
        info!("All matches are finished!");
        msg.edit(*ctx, |m| {
            m.embed(|e| {
                e.title("All matches are finished!")
                    .description("You can safely continue to next round of the tournament!")
            })
            .components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| b.label("Next Round").disabled(false).custom_id("continue"))
                })
            })
        })
        .await?;
    } else {
        error!("Some matches are not finished! Cannot continue to next round!");
        return paginate(ctx, msg, region, battles).await;
    }
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        if mci.data.custom_id.as_str() == "continue" {
            mci.defer(&ctx.http()).await?;
            update_round_config(ctx, region).await?;
            let config = crate::database::config::get_config(ctx, region).await;
            let round = config.get_i32("round").unwrap();
            msg.edit(*ctx, |m| {
                m.embed(|e| {
                    e.title("Next Round is set!")
                        .description(format!("Now the tournament is at round {round}!"))
                })
            })
            .await?;
        }
    }
    Ok(())
}

pub async fn display_false_battles(ctx: &Context<'_>, region: &Region) -> Vec<String> {
    let mut players = vec![];
    let mut result = find_all_false_battles(ctx, region).await;
    while let Some(player) = result.next().await {
        match player {
            Ok(p) => players.push(p),
            Err(err) => {
                eprintln!("Error reading document: {}", err);
                // Handle the error as needed
            }
        }
    }
    let mut match_groups: HashMap<i32, Vec<&Document>> = HashMap::new();
    for player in &players {
        if let Some(match_id) = player.get("match_id").and_then(bson::Bson::as_i32) {
            match_groups.entry(match_id).or_default().push(player);
        }
    }
    let mut battles: Vec<String> = match_groups
        .values()
        .map(|group| {
            if group.len() == 2 {
                let player1 = &group[0];
                let player2 = &group[1];
                let dis1 = player1.get_str("discord_id").unwrap_or("").to_string();
                let name1 = player1.get_str("name").unwrap_or("").to_string();
                let tag1 = player1.get_str("tag").unwrap_or("").to_string();
                let dis2 = player2.get_str("discord_id").unwrap_or("").to_string();
                let name2 = player2.get_str("name").unwrap_or("").to_string();
                let tag2 = player2.get_str("tag").unwrap_or("").to_string();
                format!(
                    r#"**Some battles are not finished!**
# Match {} 
<@{}> - <@{}>
{}({}) - {}({})"#,
                    player1.get_i32("match_id").unwrap(),
                    dis1,
                    dis2,
                    name1,
                    tag1,
                    name2,
                    tag2
                )
            } else {
                error!("{:#?}", group[0]);
                format!("Error reading match {:#?}", group[0])
            }
        })
        .collect::<Vec<String>>();
    battles.sort();
    battles
}

//I stole from poise's example
pub async fn paginate(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    pages: Vec<String>,
) -> Result<(), Error> {
    // Define some unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);
    let download_button_id = format!("{}download", ctx_id);
    let compact_id = format!("{}compact", ctx_id);

    // Send the embed with the first page as content
    let mut current_page = 0;
    msg.edit(*ctx, |b| {
        b.embed(|b| {
            b.description(pages[current_page].clone())
                .footer(|f| f.text(format!("Page {}/{}", current_page + 1, pages.len())))
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_button(|b| b.custom_id(&prev_button_id).emoji('◀'))
                    .create_button(|b| b.custom_id(&next_button_id).emoji('▶'))
                    .create_button(|b| {
                        b.custom_id(&download_button_id)
                            .label("Download")
                            .emoji('📥')
                            .disabled(true)
                    })
                    .create_button(|b| {
                        b.custom_id(&download_button_id)
                            .label("Compact")
                            .emoji('💿')
                    })
            })
        })
    })
    .await?;

    // Loop through incoming interactions with the navigation buttons
    while let Some(press) = poise::serenity_prelude::CollectComponentInteraction::new(ctx)
        // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
        // button was pressed
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout when no navigation button has been pressed for 24 hours
        .timeout(std::time::Duration::from_secs(3600 * 24))
        .await
    {
        // Depending on which button was pressed, go to next or previous page
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else if press.data.custom_id == download_button_id {
            return get_downloadable_ids(ctx, msg, region).await;
        } else if press.data.custom_id == compact_id {
            return compact(ctx, msg, region).await
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_interaction_response(ctx, |b| {
                b.kind(poise::serenity_prelude::InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|b| {
                        b.embed(|b| {
                            b.description(pages[current_page].clone()).footer(|f| {
                                f.text(format!("Page {}/{}", current_page + 1, pages.len()))
                            })
                        })
                    })
            })
            .await?;
    }

    Ok(())
}
