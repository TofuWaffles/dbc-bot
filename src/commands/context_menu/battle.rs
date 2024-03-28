use crate::commands::context_menu::player::get_data;
use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_player_by_discord_id, find_round_from_config,
};
use crate::database::open::tournament;
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::players::tournament::view2::view_opponent;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::Document;
use poise::{serenity_prelude as serenity, ReplyHandle};
const TIMEOUT: u64 = 120;

#[poise::command(context_menu_command = "View battle", guild_only)]
pub async fn view_battle(ctx: Context<'_>, user: serenity::User) -> Result<(), Error> {
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
            Some(player) => return view_battle_helper(&ctx, &msg, &region, &round, &player).await,
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

async fn view_battle_helper(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    round: &str,
    player: &Document,
) -> Result<(), Error> {
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
    let enemy = match find_enemy_by_match_id_and_self_tag(
        &ctx,
        &region,
        &round,
        &player.get_i32("match_id")?,
        &player.get_str("tag")?,
    )
    .await
    {
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
    let round = config.get_i32("round")?;
    view_opponent(&ctx, &msg, player.clone(), enemy, round, config).await
}
