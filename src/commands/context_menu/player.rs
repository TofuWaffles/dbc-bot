use crate::brawlstars::{api::request, api::APIResult, player::stat};
use crate::database::config::get_config;
use crate::database::find::{find_player_by_discord_id, find_round_from_config};
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use poise::serenity_prelude::CreateSelectMenuOption;
use poise::{serenity_prelude as serenity, ReplyHandle};
use strum::IntoEnumIterator;
use tracing::info;

const TIMEOUT: u64 = 120;
#[poise::command(context_menu_command = "Player information", guild_only)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    user: serenity::User,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| {
            s.embed(|s| {
                s.title("Getting player data...")
                    .description("How would you like to view this player data? Choose\n - Current to view the player data for the current round.\n- Custom to view the player data for a specific round.")
                    .color(0x00FF00)
            })
            .reply(true)
            .components(|c|{
                c.create_action_row(|a| {
                    a.create_button(|b| b.custom_id("current").label("Current"))
                    .create_button(|b| b.custom_id("custom").label("Custom"))
                })
            })
        })
        .await?;
    let mut cic = msg
        .clone()
        .into_message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT))
        .build();
    if let Some(mci) = cic.next().await {
        let (region, round) = match mci.data.custom_id.as_str() {
            "current" => {
                mci.defer(ctx.http()).await?;
                let roles = get_roles_from_user(&ctx, Some(&user)).await?;
                match get_region_from_role(&ctx, roles).await {
                    Some(region) => {
                        let round = find_round_from_config(&get_config(&ctx, &region).await);
                        (region, round)
                    }
                    None => {
                        return prompt(
                            &ctx,
                            &msg,
                            "Error",
                            "This player has no region role to find. Please try again with Custom.",
                            None,
                            0xFF0000,
                        )
                        .await;
                    }
                }
            }
            "custom" => {
                mci.defer(ctx.http()).await?;
                get_data(&ctx, &msg, &user).await?
            }
            _ => return Err("Invalid custom_id".into()),
        };
        let player = find_player_by_discord_id(&ctx, &region, user.id.0, round.as_str()).await?;
        match player {
            Some(player) => {
                let tag = player.get_str("tag").unwrap_or("#AAAAA");
                match request("player", tag).await {
                    Ok(APIResult::Successful(p)) => {
                        return stat(&ctx, &msg, &p, &region, Some(&player)).await
                    }
                    Ok(APIResult::NotFound(_)) => {
                        return prompt(
                            &ctx,
                            &msg,
                            "Could not find player from API",
                            "Please make sure the player tag is valid",
                            None,
                            0xFF0000,
                        )
                        .await
                    }
                    Ok(APIResult::APIError(_)) => {
                        return prompt(
                            &ctx,
                            &msg,
                            "500: Internal Server Error from",
                            "Unable to fetch player data from Brawl Stars API",
                            None,
                            0xFF0000,
                        )
                        .await
                    }
                    Err(e) => {
                        return prompt(
                            &ctx,
                            &msg,
                            "Error accessing database",
                            &format!("Error: {}", e),
                            None,
                            0xFF0000,
                        )
                        .await
                    }
                }
            }
            None => {
                return prompt(
                    &ctx,
                    &msg,
                    "404 not found",
                    "Player not found in the database",
                    None,
                    0xFF0000,
                )
                .await;
            }
        }
    }
    Ok(())
}

pub async fn get_data(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    user: &serenity::User,
) -> Result<(Region, String), Error> {
    let mut region: Option<Region> = None;
    let mut round: Option<String> = None;

    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Getting more data...")
                .description("Step 1: Please pick a region to find.")
                .color(0xFF0000)
        })
        .components(|c| {
            c.create_action_row(|a| {
                for region in Region::iter() {
                    a.create_button(|b| b.custom_id(region.short()).label(region.short()));
                }
                a
            })
        })
    })
    .await?;

    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        info!("Got interaction: {:?}", mci.data.custom_id.as_str());
        match mci.data.custom_id.as_str() {
            "APAC" | "EU" | "NASA" => {
                mci.defer(ctx.http()).await?;
                region = Region::find_key(mci.data.custom_id.as_str());
                round_getter(ctx, msg, region.as_ref().unwrap()).await?;
            }
            "confirm" => {
                mci.defer(ctx.http()).await?;
                break;
            }
            "cancel" => {
                mci.defer(ctx.http()).await?;
                return Err("User cancelled the operation".into());
            }
            "menu" => {
                mci.defer(ctx.http()).await?;
                round = Some(mci.data.values[0].clone());
                confirm(
                    ctx,
                    msg,
                    user,
                    round.as_ref().unwrap(),
                    region.as_ref().unwrap(),
                )
                .await?;
            }
            _ => {}
        }
    }
    match (region, round) {
        (Some(r), Some(rnd)) => {
            Ok((r, rnd))
        }
        (Some(r), None) => Ok((r, "".to_owned())),
        (_, _) => {
            Err("Please select a region and round".into())
        }
    }
}

async fn round_getter(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let total: i32 = find_round_from_config(&get_config(ctx, region).await)
        .split(' ')
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Getting more data...")
                .description("Please choose which round you want to search for.")
                .color(0xFF0000)
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_select_menu(|m| {
                    m.custom_id("menu")
                        .placeholder("Select a round")
                        .options(|o| {
                            for round in 1..=total {
                                let mut option = CreateSelectMenuOption::default();
                                option
                                    .label(format!("Round {}", round))
                                    .value(format!("Round {}", round));
                                o.add_option(option);
                            }
                            o
                        })
                })
            })
        })
    })
    .await?;
    Ok(())
}

async fn confirm(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    user: &serenity::User,
    round: &str,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Confirm what you are looking for")
                .description(format!(
                    r#"
**👤 Player:** {}`{}`.
**🌐 Region:** {}.    
**⚔️ Round:** {}.           
"#,
                    user.name,
                    user.id.0,
                    region.full(),
                    round
                ))
                .color(0x00FF00)
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| b.custom_id("confirm").label("Confirm"));
                a.create_button(|b| b.custom_id("cancel").label("Cancel"));
                a
            })
        })
    })
    .await?;
    Ok(())
}
