use std::collections::HashMap;

use crate::{
    bracket_tournament::config::set_config,
    bracket_tournament::{
        config::{get_config, update_round},
        region::{Mode, Region},
    },
    checks::{tournament_started, user_is_manager},
    database_utils::find_round::get_round,
    misc::{CustomError, QuoteStripper},
    Context, Error,
};
use futures::StreamExt;
use mongodb::{
    bson::doc,
    bson::{self, Document},
    Collection, Database,
};
use poise::serenity_prelude::Role;
use tracing::{info, instrument};
/// Set config for the tournament
#[poise::command(slash_command, guild_only, rename = "set-config")]
pub async fn config(
    ctx: Context<'_>,
    #[description = "Select region"] region: Region,
    #[description = "Select game mode for the tournament"] mode: Mode,
    #[description = "Set the map for that game mode"] map: Option<String>,
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
pub async fn set_manager(
    ctx: Context<'_>,
    #[description = "Select a role to hold permission to monitor the tournament"] role: Role,
) -> Result<(), Error> {
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
pub async fn set_round(
    ctx: Context<'_>,
    #[description = "Select the region"] region: Region,
    #[description = "(Optional) Set the round. By default, without this parameter, the round is increased by 1"]
    round: Option<i32>,
) -> Result<(), Error> {
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
    if !all_battles_occured(&ctx, database, &config).await? {
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

async fn all_battles_occured(
    ctx: &Context<'_>,
    database: &Database,
    config: &Document,
) -> Result<bool, Error> {
    let round = get_round(config);
    let collection = database.collection::<Document>(round.as_str());
    let mut battles = collection
        .find(
            doc! {
                "battle": false
            },
            None,
        )
        .await?;

    if battles.next().await.is_none() {
        return Ok(false);
    }

    let mut players: Vec<Document> = Vec::new();

    while let Some(player) = battles.next().await {
        match player {
            Ok(p) => players.push(p),
            Err(err) => {
                eprintln!("Error reading document: {}", err);
                // Handle the error as needed
            }
        }
    }
    let mut match_groups: HashMap<i32, Vec<&Document>> = HashMap::new();
    for player in &players {
        if let Some(match_id) = player.get("match_id").and_then(bson::Bson::as_i32) {
            match_groups
                .entry(match_id)
                .or_insert(Vec::new())
                .push(player);
        }
    }
    let ongoing: Vec<(String, String, bool)> = match_groups
        .values()
        .map(|group| {
            if group.len() == 2 {
                let player1 = &group[0];
                let player2 = &group[1];
                let name1 = player1
                    .get("discord_id")
                    .and_then(bson::Bson::as_str)
                    .unwrap_or("");
                let name2 = player2
                    .get("discord_id")
                    .and_then(bson::Bson::as_str)
                    .unwrap_or("");
                (
                    format!(
                        "Match {}: <@{}> - <@{}>",
                        player1
                            .get("match_id")
                            .and_then(bson::Bson::as_i32)
                            .unwrap_or(0),
                        name1,
                        name2
                    ),
                    "".to_string(),
                    false,
                )
            } else {
                (
                    format!("{} - {}", group[0], group[1]),
                    "".to_string(),
                    false,
                )
            }
        })
        .collect();

    ctx.send(|s| {
        s.reply(true).ephemeral(false).embed(|e| {
            e.title("**Unable to start next round due to ongoing battles!**")
                .description(format!(
                    "There are {} matches left to be completed",
                    players.len() / 2
                ))
                .fields(ongoing)
        })
    })
    .await?;

    Ok(false)
}
