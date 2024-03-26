use crate::database::config::get_config;
use crate::database::find::{find_enemy_by_match_id_and_self_tag, find_player_by_discord_id, find_round_from_config};
use crate::database::open::tournament;
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::players::tournament::view2::view_opponent;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use poise::serenity_prelude as serenity;
use strum::IntoEnumIterator;
const TIMEOUT: u64 = 120;

#[poise::command(context_menu_command = "View battle", guild_only)]
pub async fn view_battle(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.reply(true)
                .ephemeral(true)
                .embed(|e| e.title("Getting battle from this player..."))
        })
        .await?;

    let roles = get_roles_from_user(&ctx, Some(&user)).await?;
    let region = match get_region_from_role(&ctx, roles).await {
        Some(region) => region,
        None => {
            msg.edit(ctx, |m| {
                m.embed(|e| {
                    e.title("Failed to fetch user data due to lack of region role")
                        .description("Please pick a region to find.")
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
            if let Some(mci) = &cic.next().await {
                mci.defer(ctx.http()).await?;
                Region::find_key(mci.data.custom_id.as_str()).unwrap()
            } else {
                return prompt(
                    &ctx,
                    &msg,
                    "No response",
                    "No response from user",
                    None,
                    None,
                )
                .await;
            }
        }
    };

    if !tournament(&ctx, &region).await {
        return prompt(
            &ctx,
            &msg,
            "Tournament is not open!",
            "The tournament is not open yet! Please run this command again when the tournament is open!",
            None,
            None,
        )
        .await;
    }

    let round = find_round_from_config(&get_config(&ctx, &region).await);
    let player = match find_player_by_discord_id(&ctx, &region, user.id.into(), &round).await {
        Ok(user) => match user {
            Some(u) => u,
            None => {
                return prompt(
                    &ctx,
                    &msg,
                    "Not found",
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
    let enemy = match find_enemy_by_match_id_and_self_tag(
        &ctx,
        &region,
        &round,
        &player.get_i32("match_id")?,
        &player.get_str("tag")?,
    )
    .await{
        Some(e) => e,
        None => {
            return prompt(
                &ctx,
                &msg,
                "Not found",
                "Enemy not found in the database",
                None,
                None,
            )
            .await;
        }
    };
    let config = get_config(&ctx, &region).await;
    view_opponent(&ctx, &msg, player, enemy, config).await?;
    Ok(())
}
