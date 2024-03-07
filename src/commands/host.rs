use crate::{discord::{checks::is_host, menu::mod_menu}, Context, Error};
use dbc_bot::Region;
use mongodb::bson::doc;
use tracing::error;

/// Host all-in-one command
#[poise::command(slash_command, guild_only)]
pub async fn host(
    ctx: Context<'_>,
    #[description = "Select region to configurate"] region: Region,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match is_host(ctx).await {
        Ok(true) => {}
        Ok(false) => {
            ctx.send(|s| {
                s.ephemeral(true)
                    .reply(true)
                    .content("You don't have permissions to host")
            })
            .await?;
            return Ok(());
        }
        Err(e) => {
            error!("{e}");
            return Ok(());
        }
    }
    let msg = ctx
        .send(|s| {
            s.embed(|e| {
                e.title("Host-only menu")
                    .description(format!(
                        "The following mod menu is set for region: {region}"
                    ))
                    .image("")
            })
        })
        .await?;
    mod_menu(&ctx, &msg, &region, true, true, true, true).await
}
