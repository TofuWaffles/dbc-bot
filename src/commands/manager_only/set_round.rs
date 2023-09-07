use crate::{
    bracket_tournament::{
        config::{get_config, update_round},
        region::Region,
    },
    Context, Error,
};
use mongodb::bson::{doc, Document};

/// Which round are we at now?
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]

pub async fn set_round(ctx: Context<'_>, region: Region, round: Option<i32>) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    match database
        .collection::<Document>("Config")
        .update_one(config, update_round(round), None)
        .await
    {
        Ok(_) => {}
        Err(_) => {
            ctx.say("Error occurred while updating config").await?;
            return Ok(());
        }
    }

    let post_config = get_config(database).await;
    ctx.say(format!(
        "Round is set! We are at round {}",
        post_config.get("round").unwrap()
    ))
    .await?;
    Ok(())
}
