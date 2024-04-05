use crate::database::config::get_config;
use crate::database::find::find_enemy_by_match_id_and_self_tag;
use crate::discord::checks::is_host;
use crate::discord::prompt::prompt;
use crate::players::tournament::view2::view_opponent;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use mongodb::Collection;


#[poise::command(slash_command, guild_only, check = "is_host", rename = "find-match")]
pub async fn find_match(
    ctx: Context<'_>,
    #[description = "Region"] region: Region,
    #[description = "Round"] round: i32,
    #[description = "Match ID"] match_id: i32,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|m| {
            m.reply(true).ephemeral(true).embed(|e| {
                e.title("Finding match...")
                    .description("Please wait while we find the match")
                    .color(0x00FF00)
            })
        })
        .await?;
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(&ctx, &region).await;
    if round < 1 || round > config.get_i32("round")? {
        msg.edit(ctx,|m| {
            m.embed(|e| {
                e.title("Error")
                    .description("Invalid round number")
                    .color(0xFF0000)
            })
        })
        .await?;
    }
    let round_name = format!("Round {}", round);
    let collection: Collection<Document> = database.collection(&round_name);
    let player = collection
        .find_one(doc! {"match_id": match_id}, None)
        .await?;
    match player {
        None => {
            prompt(
                &ctx,
                &msg,
                "Error",
                "Match ID not found",
                None,
                Some(0xFF0000),
            )
            .await
        }
        Some(player) => {
            let tag = player.get_str("tag")?;
            let name = player.get_str("name")?;
            let id = player.get_str("discord_id")?;
            let enemy =
                find_enemy_by_match_id_and_self_tag(&ctx, &region, &round_name, &match_id, tag)
                    .await;
            match enemy {
                Some(enemy) => {
                    view_opponent(&ctx, &msg, player, enemy, round, config).await
                }
                None => {
                    let suggestion= format!("{} and {}", match_id*2, match_id*2-1);
                    prompt(
            &ctx,
            &msg,
            "Not found",
            format!("Player {name}({tag}) -`{id}` has no opponent yet! Probably run this command again to check their potential opponent from previous round with match ids: {suggestion}"),
            None,
            Some(0xFF0000),
          ).await
                }
            }
        }
    }
}
