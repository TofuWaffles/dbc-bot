use crate::{discord::menu::mod_menu, Context, Error};
use dbc_bot::Region;

// Host all-in-one command
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_MESSAGES")]
pub async fn host(
    ctx: Context<'_>,
    #[description = "Select region to configurate"] region: Region,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
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
