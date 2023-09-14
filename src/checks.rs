use crate::misc::QuoteStripper;
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Database;
use tracing::{info, instrument};

/// Checks if a tournament has started for the given region.
///
/// Make sure to pass in the database that corresponds to the region you want to check.
pub async fn tournament_started(database: &Database) -> Result<bool, Error> {
    let config = database
        .collection::<Document>("Config")
        .find_one(None, None)
        .await?
        .unwrap();

    let tournament_started = config.get_bool("tournament_started")?;

    Ok(tournament_started)
}

/// Checks if the user is a manager. Returns true if they are, false otherwise.
/// The bot owner may set new managers using the /set-manager command
///
/// Simply stick this at the top of your command to implement this check:
/// ```
/// if !user_is_manager(ctx).await? { return Ok(()) }
/// ```
#[instrument]
pub async fn user_is_manager(ctx: Context<'_>) -> Result<bool, Error> {
    info!("Checking permissions...");
    let guild_id = ctx.guild_id().unwrap().to_string();
    let database = &ctx.data().database.general;
  
    let mut managers = database
        .collection::<Document>("Managers")
        .find(doc! {"guild_id": &guild_id}, None)
        .await?;
    while let Some(manager) = managers.try_next().await?{
        let role_id = manager.get("role_id").unwrap().to_string().strip_quote();
        if ctx.author().has_role(ctx.http(), guild_id.parse::<u64>().unwrap(), role_id.parse::<u64>().unwrap()).await?{
            return Ok(true)
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
