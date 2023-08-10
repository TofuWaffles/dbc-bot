use crate::utils::types;

// A simple ping command as an example. You might want to disable this in production.
#[poise::command(slash_command)]
pub async fn ping(ctx: types::Context<'_>) -> Result<(), types::Error> {
    let response = "Pong".to_owned();
    ctx.say(response).await?;
    Ok(())
}
