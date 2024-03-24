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
    info!(
        "RUNNING context-menu 'Player information' on {}({})",
        user.name, user.id.0
    );
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| {
            s.embed(|s| {
                s.title("Getting player data...")
                    .description("Please wait a moment")
                    .color(0x00FF00)
            })
            .reply(true)
        }).await?;
    
    let (mut region, mut round) = (None, None);
    let roles = get_roles_from_user(&ctx, Some(&user)).await?;
    match get_region_from_role(&ctx, roles).await {
        Some(r) => region = Some(r),
        None => {
            let (r, rnd) = get_data(&ctx, &msg, &user).await?;
            match rnd.as_str(){
                "" => round = Some(find_round_from_config(&get_config(&ctx, &r).await)),
                _ => round = Some(rnd)
            }
            region = Some(r);
        }
    }
    if round.is_none(){
        round = Some(find_round_from_config(&get_config(&ctx, region.as_ref().unwrap()).await));
    }
    let id: u64 = user.id.into();

    let player_from_db = match find_player_by_discord_id(
        &ctx,
        region.as_ref().unwrap(),
        id,
        &round.unwrap(),
    )
    .await
    {
        Ok(player) => match player {
            Some(p) => p,
            None => {
                return prompt(
                    &ctx,
                    &msg,
                    "404 not found",
                    "Player not found in the database",
                    None,
                    None,
                )
                .await;
            }
        },
        Err(_) => {
            return prompt(
                &ctx,
                &msg,
                "Error accessing database",
                "Please try again later",
                None,
                None,
            )
            .await;
        }
    };
    let player = request("player", player_from_db.get_str("tag").unwrap()).await?;
    match player {
        APIResult::Successful(p) => {
            stat(&ctx, &msg, &p, &region.unwrap(), Some(&player_from_db)).await
        }
        APIResult::NotFound(_) => {
            prompt(
                &ctx,
                &msg,
                "Could not find player from API",
                "Please make sure the player tag is valid",
                None,
                None,
            )
            .await
        }
        APIResult::APIError(_) => {
            prompt(
                &ctx,
                &msg,
                "500: Internal Server Error from",
                "Unable to fetch player data from Brawl Stars API",
                None,
                None,
            )
            .await
        }
    }
}

async fn get_data(ctx: &Context<'_>, msg: &ReplyHandle<'_>, user: &serenity::User) -> Result<(Region, String), Error> {
    let mut region: Option<Region> = None;
    let mut round: Option<String> = None;

    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Failed to fetch user data due to lack of region role!")
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
            "menu"=> {
                mci.defer(ctx.http()).await?;
                round = Some(mci.data.values[0].clone());
                confirm(ctx, msg, &user, &round.as_ref().unwrap(), &region.as_ref().unwrap()).await?;                
            }
            _ => {

            }
        }
    }
    match (region, round) {
        (Some(r), Some(rnd)) => {
            return Ok((r, rnd));
        }
        (Some(r), None) => return Ok((r, "".to_owned())),
        (_, _) => {
            return Err("Please select a region and round".into());
        }
    }
        
}

async fn round_getter(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    let total: i32 = find_round_from_config(&get_config(&ctx, region).await)
        .split(" ")
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Failed to fetch user data due to lack of region role!")
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

async fn confirm(ctx: &Context<'_>, msg: &ReplyHandle<'_>, user: &serenity::User, round: &str, region: &Region) -> Result<(), Error>{
    msg.edit(*ctx, |m|{
        m.embed(|e|{
            e.title("Confirm what you are looking for")
                .description(format!(r#"
**👤 Player:** {}`{}`.
**🌐 Region:** {}.    
**⚔️ Round:** {}.           
"#, user.name, user.id.0, region.full(), round))
                .color(0x00FF00)
        })
        .components(|c|{
            c.create_action_row(|a|{
                a.create_button(|b| b.custom_id("confirm").label("Confirm"));
                a.create_button(|b| b.custom_id("cancel").label("Cancel"));
                a
            })
        })
    }).await?;
    Ok(())
}