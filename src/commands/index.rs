use crate::database::battle::is_battle;
use crate::database::config::get_config;
use crate::database::find::{find_round_from_config, find_self_by_discord_id};
use crate::database::open::{all_tournaments, registration};
use crate::discord::menu::registration_menu;
use crate::discord::menu::tournament_menu;
use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;
const DELAY: u64 = 1;

// Tournament all-in-one command
#[poise::command(slash_command)]
pub async fn index(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    return home(ctx, None).await;
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

    if all_tournaments(&ctx).await {
        match find_self_by_discord_id(&ctx).await? {
            Some(player) => {
                if is_battle(
                    &ctx,
                    player.get("tag").unwrap().as_str(),
                    find_round_from_config(
                        &get_config(
                            &ctx,
                            Region::from_bson(player.get("region").unwrap())
                                .as_ref()
                                .unwrap(),
                        )
                        .await,
                    ),
                )
                .await?
                {
                    return tournament_menu(&ctx, &msg, true, true, true, true, Some(player)).await;
                } else {
                    return tournament_menu(&ctx, &msg, false, true, false, false, None).await;
                };
            }
            None => {
                return Ok(prompt(
                    &ctx,
                    &msg,
                    "You did not register for the tournament!",
                    "The tournament has already started, and you did not register in time...",
                    None,
                    None,
                )
                .await?)
            }
        }
    } else {
        if registration(&ctx).await {
            match find_self_by_discord_id(&ctx).await? {
                Some(player) => {
                    return registration_menu(&ctx, &msg, false, true, true, true, Some(player))
                        .await;
                }
                None => {
                    return registration_menu(&ctx, &msg, true, false, false, true, None).await;
                }
            }
        } else {
            todo!()
        }
    }
}
