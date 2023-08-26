use crate::{Context, Error};
use crate::bracket_tournament::api;
use crate::utils::misc::QuoteStripper;

/// Sign up for Discord Brawl Cup Tournament!
#[poise::command(slash_command, prefix_command,)]
async fn registry(
  ctx: Context<'_>, 
  #[description = "Put your player tag here (without #)"] tag: String,
  #[description = "Put your region here"] ) -> Result<(), Error>{
  }