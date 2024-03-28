use crate::{
    database::find::find_tag,
    discord::role::{get_region_from_role, get_roles_from_user},
    Context, Error,
};
use crate::{
    database::find::{find_player_by_discord_id, find_player_by_discord_id_without_region},
    discord::{
        checks::is_host,
        prompt::{self, prompt},
    },
    players::view::view_info,
};
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::{User, UserId};
use poise::ReplyHandle;
use tracing::{error, span, Level};
/// Lookup player by tag or user
#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn lookup_player(
    ctx: Context<'_>,
    #[description = "Player tag"] player_tag: Option<String>,
    #[description = "User id"] user_id: Option<String>,
) -> Result<(), Error> {
    span!(Level::INFO, "lookup_player", player_tag);
    let msg = ctx
        .send(|s| {
            s.embed(|e| e.title("Looking up player"))
                .ephemeral(true)
                .reply(true)
        })
        .await?;
    // We probably don't need this. I'll give it another look later. - Doof
    match (player_tag, user_id) {
        (Some(tag), None) => {
            if let Some(player) = find_tag(&ctx, &tag).await {
                view_info(&ctx, &msg, player).await
            } else {
                prompt(
                    &ctx,
                    &msg,
                    "Cannot find player with this tag",
                    "Unable to find this tag from any regions!",
                    None,
                    Some(0xFF0000),
                )
                .await
            }
        }
        (None, Some(user_id)) => {
            let user = match UserId(user_id.parse::<u64>().unwrap_or(0))
                .to_user(ctx.http())
                .await
            {
                Ok(u) => u,
                Err(_) => {
                    prompt(
                        &ctx,
                        &msg,
                        "Cannot find user",
                        "Make sure you have the correct user id and try again",
                        None,
                        Some(0xFF0000),
                    )
                    .await?;
                    return Err("Cannot find user".into());
                }
            };
            match analyze_id_and_find_player(&ctx, &msg, user).await {
                Ok(player) => {
                    if let Some(player) = player {
                        view_info(&ctx, &msg, player).await
                    } else {
                        prompt(
                            &ctx,
                            &msg,
                            "Cannot find player with this discord id",
                            "Unable to find this discord id from any regions!",
                            None,
                            Some(0xFF0000),
                        )
                        .await
                    }
                }
                Err(e) => {
                    error!("{e}");
                    prompt(
                        &ctx,
                        &msg,
                        "Cannot find player with this discord id",
                        "Unable to find this discord id from any regions!",
                        None,
                        Some(0xFF0000),
                    )
                    .await
                }
            }
        }
        (None, None) => {
            prompt(
                &ctx,
                &msg,
                "Cannot search for player",
                "Please provide either a player tag or a discord user to search",
                None,
                Some(0xFF0000),
            )
            .await
        }
        (Some(_), Some(_)) => {
            prompt(
                &ctx,
                &msg,
                "The developers are lazy to handle this case",
                "Why would you do this to us :c. One parameter is enough!",
                None,
                Some(0xFF0000),
            )
            .await
        }
    }
}

async fn analyze_id_and_find_player(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    user: User,
) -> Result<Option<Document>, Error> {
    let user_id = user.id.0;
    let roles = match get_roles_from_user(ctx, Some(&user)).await {
        Ok(roles) => roles,
        Err(_) => {
            prompt(
                ctx,
                msg,
                "Cannot get user roles",
                "The user is not in the server! Trying to find the user in the database...",
                None,
                Some(0xFF0000),
            )
            .await?;
            return find_player_by_discord_id_without_region(ctx, user_id).await;
        }
    };
    let region = match get_region_from_role(ctx, roles).await {
        Some(region) => region,
        None => {
            msg.edit(*ctx, |s| {
                s.reply(true)
                    .ephemeral(true)
                    .embed(|e| e.title("Failed to get user region"))
            })
            .await?;
            return Err("Failed to get user region".into());
        }
    };
    find_player_by_discord_id(ctx, &region, user_id, "Players").await
}
