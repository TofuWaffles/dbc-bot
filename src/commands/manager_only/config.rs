use crate::{
    bracket_tournament::config::set_config,
    bracket_tournament::region::{Mode, Region},
    checks::user_is_manager,
    misc::{CustomError, QuoteStripper},
    Context, Error,
};
use mongodb::{bson::doc, bson::Document, Collection};

/// Set config for the tournament
#[poise::command(slash_command, guild_only)]
pub async fn config(
    ctx: Context<'_>,
    region: Region,
    mode: Mode,
    map: Option<String>,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    let config = set_config(&mode, map.as_ref());
    match collection.update_one(doc! {}, config, None).await {
        Ok(_) => {}
        Err(_) => {
            return Err(Box::new(CustomError(
                "Error occurred while updating config".to_string(),
            )))
        }
    };
    let post_config = match collection.find_one(doc! {}, None).await {
        Ok(Some(config)) => config,
        Ok(None) => return Err(Box::new(CustomError("Config not found".to_string()))),
        Err(_) => {
            return Err(Box::new(CustomError(
                "Error occurred while finding config".to_string(),
            )))
        }
    };
    let mut printed_config: Vec<(String, String, bool)> = vec![];
    for (key, value) in post_config.iter() {
        printed_config.push((
            format!("**{}**: {}", key, value.to_string().strip_quote()),
            "".to_string(),
            false,
        ))
    }
    printed_config.remove(0); //remove ObjectID to print lol
    ctx.send(|s| {
        s.reply(true).ephemeral(true).embed(|e| {
            e.title("**Configuration has been updated!**")
                .description(format!(
                    "The configuration for {} tournament is shown below",
                    region
                ))
                .fields(printed_config)
        })
    })
    .await?;
    Ok(())
}
