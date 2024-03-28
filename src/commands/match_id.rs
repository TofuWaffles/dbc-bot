use crate::database::config::get_config;
use crate::database::find::find_enemy_by_match_id_and_self_tag;
use crate::discord::checks::is_host;
use crate::discord::prompt::prompt;
use crate::players::tournament::view2::view_opponent;
use crate::{Context, Error};
use crate::Region;
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use poise::ReplyHandle;

#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn find_match(
  ctx: Context<'_>,
  #[description = "Region"] region: Region, 
  #[description = "Round"] round: i32,
  #[description = "Match ID"] match_id: i32,
) -> Result<(), Error> {
  ctx.defer_ephemeral().await?;
  let msg = ctx.send(|m|{
    m.reply(true)
      .ephemeral(true)
      .embed(|e|{
        e.title("Finding match...")
          .description("Please wait while we find the match")
          .color(0x00FF00)
  })}).await?;
  let database = ctx.data().database.regional_databases.get(&region).unwrap();
  let config = get_config(&ctx, &region).await;
  if round < 1 || round > config.get_i32("round")?{
    ctx.send(|m| m.embed(|e|{
      e.title("Error")
      .description("Invalid round number")
      .color(0xFF0000)
    })).await?;
  }
  let round_name = format!("Round {}", round);
  let collection: Collection<Document> = database.collection(&round_name);
  let player = collection.find_one(doc!{"match_id": match_id}, None).await?;
  match player{
    None => {
      return prompt(
        &ctx,
        &msg,
        "Error",
        "Match ID not found",
        None,
        Some(0xFF0000),
      ).await;
    }
    Some(player) => {
      let tag = player.get_str("tag")?;
      let enemy = find_enemy_by_match_id_and_self_tag(&ctx, &region, &round_name, &match_id, tag).await;
      match enemy{
        Some(enemy) =>{ 
          return view_opponent(&ctx, &msg, player, enemy, round, config).await
        }
        None => {
          return prompt(
            &ctx,
            &msg,
            "Not found",
            "This player has no opponent yet! Probably run this command again to check previous round with twice the value of match id.",
            None,
            Some(0xFF0000),
          ).await;
        }
      }
    }
  }
}

