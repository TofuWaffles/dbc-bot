use crate::{Context, Error};

/// Ping the bot and possibly get a response, probably, maybe, maybe not?
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Pong".to_owned();
    ctx.say(response).await?;
    Ok(())
}