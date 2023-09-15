use crate::{
    bracket_tournament::config::set_config,
    bracket_tournament::{region::{Mode, Region}, config::{get_config, update_round}},
    checks::{user_is_manager, tournament_started},
    misc::{CustomError, QuoteStripper},
    Context, Error,
};
use mongodb::{bson::doc, bson::Document, Collection, Database};
use poise::serenity_prelude::Role;
use tracing::{info, instrument};
/// Set config for the tournament
#[poise::command(
    slash_command, 
    guild_only,
    rename = "set-config",
)]
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
/////////////////////////////////////////////////////
/// Set a role as a manager to access manager-only commands. Only the bot owner can run this.
#[instrument]
#[poise::command(slash_command, guild_only, owners_only, rename = "set-manager")]
pub async fn set_manager(ctx: Context<'_>, role: Role) -> Result<(), Error> {
    info!("Setting manager for {}", role);
    let database = &ctx.data().database.general;
    let guild_id = ctx.guild_id().unwrap().to_string();
    let guild_name = ctx.guild().unwrap().name;
    let role_id = role.id.to_string();
    let role_name = role.name;

    if role_exists(database, &guild_id, &role_id).await? {
        ctx.send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content(format!("{} is already a manager!", &role_name))
        })
        .await?;
    } else {
        let collection = database.collection::<Document>("Managers");
        let new_role: Document = doc! {
            "guild_id": &guild_id,
            "guild_name": &guild_name,
            "role_id": &role_id,
            "role_name": &role_name,
        };
        collection.insert_one(new_role, None).await?;
        ctx.send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content(format!("{} is now a manager!", &role_name))
        })
        .await?;
    };
    Ok(())
}

async fn role_exists(
    database: &Database,
    guild_id: &String,
    role_id: &String,
) -> Result<bool, Error> {
    let collection = database.collection::<Document>("Managers");
    match collection
        .find_one(
            doc! {
                "guild_id": guild_id,
                "role_id": role_id
            },
            None,
        )
        .await
    {
        Ok(Some(_)) => Ok(true),    
        Ok(None) => Ok(false),       
        Err(err) => Err(err.into()),
    }
}
/////////////////////////////////////////////////////////////////
/// Get the current round of the tournament
#[poise::command(slash_command, guild_only)]
pub async fn set_round(ctx: Context<'_>, region: Region, round: Option<i32>) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    if !tournament_started(database).await? {
        ctx.send(|s| {
            s.ephemeral(true)
            .reply(true)
            .content("Unable to set the round for the current tournament: the tournament has not started yet!")
        })
        .await?;
        return Ok(());
    }

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
    match sort_collection(database, &post_config).await {
        Ok(_) => {}
        Err(_) => {
            ctx.send(|s| {
                s.content("Error occurred while sorting collection")
                    .ephemeral(true)
                    .reply(true)
            })
            .await?;
            return Ok(());
        }
    };
    ctx.send(|s| {
        s.ephemeral(true).reply(true).embed(|e| {
            e.title("Round is set successfully!").description(format!(
                "Round is set! We are at round {}",
                post_config.get("round").unwrap()
            ))
        })
    })
    .await?;
    Ok(())
}

async fn sort_collection(database: &Database, config: &Document) -> Result<(), Error> {
    let round = config.get("round").unwrap();
    let collection = database.collection::<Document>(format!("Round{}", round).as_str());
    let pipeline = vec![doc! {
        "$sort": {
            "match_id": 1
        }
    }];
    collection.aggregate(pipeline, None).await?;
    Ok(())
}

