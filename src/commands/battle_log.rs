use crate::bracket_tournament::battle_log::BattleLog;
use crate::{Context, Error};

/// Get the latest battle log ofc the player
#[poise::command(slash_command, prefix_command)]
pub async fn log(
  ctx:Context<'_>, 
  #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), Error> {
  let log = BattleLog::new(&tag).await;
  let response = log.get_latest_log();
  ctx.say(response).await?;
  Ok(())
}