use crate::utils::types;

/// Ping the bot and possibly get a response, probably, maybe, maybe not?
#[poise::command(slash_command)]
pub async fn ping(ctx: types::Context<'_>) -> Result<(), types::Error> {
    let response = "Pong".to_owned();
    ctx.say(response).await?;
    Ok(())
}