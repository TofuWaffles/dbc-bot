use crate::utils::types;
use crate::bracket_tournament::battle_log::BattleLog;


/// Get the latest battle log ofc the player
#[poise::command(slash_command, prefix_command)]
pub async fn log(
  ctx: types::Context<'_>, 
  #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), types::Error> {
  let log = BattleLog::new(&tag).await;
  let response = log.get_latest_log();
  ctx.say(response).await?;
  Ok(())
}