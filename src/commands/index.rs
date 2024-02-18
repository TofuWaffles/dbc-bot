use crate::database::battle::is_battle;
use crate::database::config::get_config;
use crate::database::find::{find_round_from_config, find_self_by_discord_id};
use crate::database::open::{registration_open, registration_region_open};
use crate::discord::menu::registration_menu;
use crate::discord::menu::tournament_menu;
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::{Context, Error};
use poise::ReplyHandle;
use tracing::info;
const DELAY: u64 = 1;

// Tournament all-in-one command
#[poise::command(slash_command, guild_only)]
pub async fn index(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    home(ctx, None).await
}

pub async fn home(ctx: Context<'_>, msg: Option<ReplyHandle<'_>>) -> Result<(), Error> {
    let msg = match msg {
        Some(msg) => msg,
        None => {
            ctx.send(|s| {
                s.embed(|e| {
                    e.title("Menu")
                    .description("Welcome to the menu! Here you can find commands that are available for you!\nRedirecting you to menu...")
                    .image("")
                })
            })
            .await?
        }
    };

    std::thread::sleep(std::time::Duration::from_secs(DELAY));

    // Checking participation status with regional roles
    // Found role => check registration status to display either registration menu or tournament menu
    // No role => check registration status to display either register or nothing
    let roles = get_roles_from_user(&ctx, None).await.unwrap();
    let region = get_region_from_role(&ctx, roles);
    match region {
        Some(region) => {
            let player = match find_self_by_discord_id(&ctx).await.unwrap() {
                Some(player) => player,
                None => {
                    prompt(
                        &ctx,
                        &msg,
                        "You did not register for the tournament!",
                        "The tournament has already started, and you did not register in time...",
                        None,
                        None,
                    )
                    .await?;
                    return Ok(());
                }
            };
            if registration_region_open(&ctx, &region).await {
                registration_menu(&ctx, &msg, false, true, true, true, Some(player)).await
            } else {
                match find_self_by_discord_id(&ctx).await.unwrap() {
                    Some(player) => {
                        if !is_battle(
                            &ctx,
                            player.get("tag").unwrap().as_str(),
                            find_round_from_config(&get_config(&ctx, &region).await),
                        )
                        .await?
                        { 
                            info!("{} has not done any battle in the current round!", player.get_str("tag").unwrap());
                            tournament_menu(&ctx, &msg, true, true, true, true).await
                        } else {
                            info!("{} has done battle in the current round!", player.get_str("tag").unwrap());
                            tournament_menu(&ctx, &msg, false, true, false, false).await
                        }
                    }
                    None => prompt(
                        &ctx,
                        &msg,
                        "You did not register for the tournament!",
                        "The tournament has already started, and you did not register in time...",
                        None,
                        None,
                    )
                    .await,
                }
            }
        }
        None => {
            if registration_open(&ctx).await {
                registration_menu(&ctx, &msg, true, false, false, true, None).await
            } else {
                prompt(
                    &ctx,
                    &msg,
                    "You did not register for the tournament!",
                    "The tournament has already started, and you did not register in time...",
                    None,
                    None,
                )
                .await
            }
        }
    }
}
