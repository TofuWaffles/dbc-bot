use dbc_bot::QuoteStripper;
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use tracing::{info, instrument};
pub async fn user_is_manager(ctx: Context<'_>) -> Result<bool, Error> {
    info!("Checking permissions...");
    let guild_id = ctx.guild_id().unwrap().to_string();
    let database = &ctx.data().database.general;

    let mut managers = database
        .collection::<Document>("Managers")
        .find(doc! {"guild_id": &guild_id}, None)
        .await?;
    while let Some(manager) = managers.try_next().await? {
        let role_id = manager.get("role_id").unwrap().to_string().strip_quote();
        if ctx
            .author()
            .has_role(
                ctx.http(),
                guild_id.parse::<u64>().unwrap(),
                role_id.parse::<u64>().unwrap(),
            )
            .await?
        {
            return Ok(true);
        }
    }
    ctx.send(|s| {
        s.content("Sorry, you do not have the permissions required to run this command!")
            .ephemeral(true)
            .reply(true)
    })
    .await?;
    Ok(false)
}
