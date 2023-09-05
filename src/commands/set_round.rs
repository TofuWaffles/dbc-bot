use crate::bracket_tournament::config::{get_config, update_round};
use crate::bracket_tournament::region::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use tracing::instrument;

/// Sets the current round for the tournament
#[instrument]
#[poise::command(slash_command)]
pub async fn set_round(ctx: Context<'_>, region: Region, round: Option<i32>) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    database
        .collection::<Document>("Config")
        .update_one(config, update_round(round), None)
        .await
        .unwrap();
    ctx.say("Round is set!").await?;
    Ok(())
}
