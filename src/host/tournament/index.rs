use crate::database::config::get_config;
use crate::database::find::find_round_from_config;
use crate::database::stat::count_registers;
use crate::Context;
use crate::Error;
use dbc_bot::Region;
use futures::StreamExt;
use poise::serenity_prelude::ReactionType;
use poise::ReplyHandle;

use super::disqualify::disqualify_players;
use super::next::display_next_round;
use super::reset::reset;
use super::setup::start_tournament;
const TIMEOUT: u64 = 300;

pub async fn tournament_mod_panel(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    display_start_menu(ctx, msg, region).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "start" => {
                mci.defer(&ctx.http()).await?;
                return start_tournament(ctx, msg, region).await;
            }
            "next" => {
                mci.defer(&ctx.http()).await?;
                return display_next_round(ctx, msg, region).await;
            }
            "disqualify" => {
                mci.defer(&ctx.http()).await?;
                return disqualify_players(ctx, msg, region).await;
            }
            "reset" => {
                mci.defer(&ctx.http()).await?;
                return reset(ctx, msg, Some(region)).await;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn display_start_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let round = find_round_from_config(&get_config(ctx, region).await);
    let valid = tournament_available(ctx, region).await || prerequisite(ctx, region).await;
    match round.as_str() {
        "Players" => {
            let count_prompt = format!(
                "There are {} registers for this region!",
                count_registers(ctx, region).await?
            );
            let valid_prompt = match &valid{
          true => "All configurations are set! You can start tournament now",
          false => "Some configurations are missing, please re-run </host:1185308022285799546> and check ⚙️ configuration menu",
        };
            display_start_buttons(ctx, msg, &valid, &false).await?;
            msg.edit(*ctx, |m| {
                m.embed(|e| {
                    e.title("**Tournament menu**")
                        .description(format!("{}\n{}\n", count_prompt, valid_prompt))
                })
            })
            .await?;
        }
        _ => {
            display_start_buttons(ctx, msg, &false, &true).await?;
            msg.edit(*ctx, |m| {
                m.embed(|e| {
                    e.title("Tournament menu")
                        .description(format!("Tournament is at {}!", round))
                })
            })
            .await?;
        }
    }
    Ok(())
}
async fn display_start_buttons(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    start: &bool,
    next: &bool,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.components(|c| {
            c.create_action_row(|row| {
                row.create_button(|b| {
                    b.custom_id("start")
                        .label("Start tournament")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("▶️".to_string()))
                        .disabled(!start)
                });
                row.create_button(|b| {
                    b.custom_id("next")
                        .label("Next round")
                        .emoji(ReactionType::Unicode("▶️".to_string()))
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .disabled(!next)
                })
            })
            .create_action_row(|row| {
                row.create_button(|b| {
                    b.custom_id("disqualify")
                        .label("Disqualify players")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🔨".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("reset")
                        .label("Reset tournament")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🔨".to_string()))
                })
            })
        })
    })
    .await?;
    Ok(())
}
async fn prerequisite(ctx: &Context<'_>, region: &Region) -> bool {
    let config = get_config(ctx, region).await;
    !(config.get("mode").is_none()
        || config.get("role").is_none()
        || config.get("channel").is_none()
        || config.get("bracket_channel").is_none())
}

async fn tournament_available(ctx: &Context<'_>, region: &Region) -> bool {
    let config = get_config(ctx, region).await;
    !config.get_bool("tournament").unwrap()
}
