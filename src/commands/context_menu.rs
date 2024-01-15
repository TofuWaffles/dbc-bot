use crate::brawlstars::{api::request, api::APIResult, player::stat};
use crate::database::find::find_player_by_discord_id;
use crate::discord::prompt::prompt;
use crate::discord::role::{get_region_from_role, get_roles_from_user};
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tracing::info;

#[poise::command(context_menu_command = "Player information", guild_only)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] user: serenity::User,
) -> Result<(), Error> {
    info!("Getting participant data");
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| s.content("Getting player info...").reply(true))
        .await?;
    let roles = match get_roles_from_user(&ctx, Some(&user)).await{
        Some(roles) => roles,
        None => {
            return prompt(
                &ctx, 
                &msg, 
                "Failed to fetch roles from user", 
                "Please only use this feature in a server", 
                None, 
                None
            ).await;
        }
    };
    let region = match get_region_from_role(&ctx, roles).await {
        Some(region) => region,
        None => {
            return prompt(
                &ctx,
                &msg,
                "Failed to fetch user data due to lacking of region role",
                "Please make sure the user have a region role",
                None,
                None,
            )
            .await;
        }
    };
    let id: u64 = user.id.into();
    let player_from_db = match find_player_by_discord_id(&ctx, &region, id).await {
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
            stat(&ctx, &msg, &p, &region).await
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
