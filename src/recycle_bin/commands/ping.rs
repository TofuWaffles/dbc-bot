use crate::checks::user_is_manager;
use crate::{Context, Error};

/// Test this command to see if it works
#[poise::command(slash_command, guild_only)]
pub async fn test(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let something = ctx.send(|s| s.content("test")).await?;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    something.edit(ctx, |m| m.content("test2")).await?;
    Ok(())
}
