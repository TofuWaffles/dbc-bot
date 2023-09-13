use crate::checks::user_is_manager;
use crate::{Context, Error};

/// Ping the bot and possibly get a response, probably, maybe, maybe not?
#[poise::command(slash_command, guild_only)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let response = "Pong".to_string();
    ctx.say(response).await?;
    Ok(())
}
