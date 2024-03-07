use dbc_bot::{CustomError, Region};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude::User;
use strum::IntoEnumIterator as _;
use tracing::{error, span, Level};

use crate::{database::find::find_tag, Context, Error};
use crate::discord::checks::is_host;
/// Lookup player by tag or user
#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn lookup_player(
    ctx: Context<'_>,
    player_tag: Option<String>,
    user: Option<User>
) -> Result<(), Error> {
    span!(Level::INFO, "lookup_player", player_tag);

    // We probably don't need this. I'll give it another look later. - Doof
    if player_tag.is_none() && user.is_none() {
        ctx.send(|s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("Please provide either a player tag or a discord user to search")
            })
        })
        .await?;

        return Ok(());
    }

    match player_tag {
        Some(tag) => match find_tag(&ctx, &tag).await {
            Some(player) => {
                let id = match player.get_str("discord_id") {
                    Err(_) => {
                        ctx.send(|s| {
                            s.reply(true)
                                .ephemeral(true)
                                .embed(|e| e.title("Failed to get Discord id"))
                        })
                        .await?;
                        return Err(Box::new(CustomError("Failed to get user id".to_string())));
                    }
                    Ok(id) => id,
                };

                ctx.send(|s| {
                    s.reply(true).ephemeral(true).embed(|e| {
                        e.title("Player found").description(format!(
                            "Player <@{}> found with in-game tag: {}",
                            id, tag
                        ))
                    })
                })
                .await?;
                return Ok(());
            }
            None => {
                ctx.send(|s| {
                    s.reply(true)
                        .ephemeral(true)
                        .embed(|e| e.title("Player not found"))
                })
                .await?;
                return Ok(());
            }
        },
        None => (),
    };

    match user {
        Some(user) => {
            let user_id = user.id.as_u64().to_string();

            for region in Region::iter() {
                let database = ctx.data().database.regional_databases.get(&region).unwrap();
                let player_data: Collection<Document> = database.collection("Players");
                match player_data
                    .find_one(doc! {"discord_id": user_id.clone()}, None)
                    .await?
                {
                    Some(player_doc) => {
                        let tag = match player_doc.get_str("tag") {
                            Ok(tag) => tag,
                            Err(_) => {
                                ctx.send(|s| {
                                    s.reply(true)
                                        .ephemeral(true)
                                        .embed(|e| e.title("Failed to get player tag"))
                                })
                                .await?;
                                return Err(Box::new(CustomError(
                                    "Failed to get player tag".to_string(),
                                )));
                            }
                        };
                        ctx.send(|s| {
                            s.reply(true).ephemeral(true).embed(|e| {
                                e.title("Player found").description(format!(
                                    "Player <@{}> found with in-game tag: {}",
                                    user_id, tag
                                ))
                            })
                        })
                        .await?;
                        return Ok(());
                    }
                    None => todo!(),
                }
            }
            ctx.send(|s| {
                s.reply(true)
                    .ephemeral(true)
                    .embed(|e| e.title("Player not found"))
            })
            .await?;
            return Ok(());
        }
        None => {
            error!("Neither the player tag nor the user were provided",);
            return Err(Box::new(CustomError(
                "Neither the player tag nor the user were provided".to_string(),
            )));
        }
    }
}
