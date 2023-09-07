use crate::bracket_tournament::config::start_tournament_config;
use crate::bracket_tournament::{region::Region, *};
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Bson::Null, Document},
    options::AggregateOptions,
    Collection,
};
use strum::IntoEnumIterator;
use tracing::{info, instrument};
const MINIMUM_PLAYERS: i32 = 3; // The minimum amount of players required to start a tournament

///Run this command once registration closes to start the tournament.
#[instrument]
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
    rename = "start-tournament"
)]
pub async fn start_tournament(ctx: Context<'_>) -> Result<(), Error> {
    info!("Attempting to start the tournament...");

    let mut started_tournaments = Vec::<Region>::new();
    // Handling each region mathematical computations to preset brackets
    ctx.say("Starting tournament...").await?;
    for region in Region::iter() {
        info!("Starting tournament for region {}", region);
        let database = ctx.data().database.regional_databases.get(&region).unwrap();

        // Disable registration
        let config = match database
            .collection::<Document>("Config")
            .find_one(None, None)
            .await
        {
            Ok(Some(config)) => config,
            Ok(None) => {
                ctx.say(format!("Config for {} not found", region)).await?;
                continue;
            }
            Err(_) => {
                ctx.say(format!(
                    "Error occurred while finding config for {}",
                    region
                ))
                .await?;
                continue;
            }
        };
        if !is_config_ok(&ctx, &config, &region).await? {
            continue;
        }
        database
            .collection::<Document>("Config")
            .update_one(config, start_tournament_config(), None)
            .await?;

        // Counting players in a region
        let collection: Collection<Document> = database.collection("Player");
        let count = collection.count_documents(None, None).await? as i32;

        // If there aren't enough players in a region, skip to next region
        if count < MINIMUM_PLAYERS {
            ctx.say(
                format!(
                    "Tournament for {} cannot start due to having only {} players (at least {} are needed)",
                    region,
                    count,
                    MINIMUM_PLAYERS
                )
                .as_str(),
            )
            .await?;
            continue;
        }
        let rounds = (count as f64).log2().ceil() as u32;
        let byes = 2_i32.pow(rounds) - count;
        info!(
            "Generating a bracket tournament with {} rounds and {} byes",
            rounds, byes
        );
        ctx.channel_id()
            .send_message(ctx, |m| {
                m.content(format!("There are {} byes in region {}", byes, region))
            })
            .await?;
        match byes {
            0 => {}
            _ => {
                for bye in 1..=byes {
                    let mannequin = mannequin::add_mannequin(&region, Some(bye), None);
                    collection.insert_one(mannequin, None).await?;
                }
            }
        }
        assign_match_id::assign_match_id(&region, database, byes).await?;
        //Create rounds collection for each databases
        info!("Writing round collections to the databases");
        for round in 1..=rounds {
            let collection_names = format!("Round {}", round);
            if !database
                .list_collection_names(None)
                .await
                .unwrap()
                .contains(&collection_names)
            {
                database
                    .create_collection(format!("Round {}", round), None)
                    .await?;
            }
        }

        //Clone and sort all player data to round 1
        let pipeline = vec![
            doc! {
                "$sort": {
                    "match_id": 1
                }
            },
            doc! {
                "$out": "Round 1"
            },
        ];
        let aggregation_options = AggregateOptions::builder().allow_disk_use(true).build();

        // Run the aggregation pipeline to copy and sort documents
        collection
            .aggregate(pipeline, Some(aggregation_options))
            .await?;

        started_tournaments.push(region);
    }

    if started_tournaments.is_empty() {
        info!("No tournaments have been started");
        ctx.channel_id()
            .send_message(ctx, |m| m.content("No tournaments have been started"))
            .await?;
    } else {
        info!("Tournament(s) successfully started!");
        ctx.channel_id()
            .send_message(ctx, |m| {
                m.content(format!(
                    "Tournament started for regions: {:#?}",
                    started_tournaments
                ))
            })
            .await?;
    }
    Ok(())
}

async fn is_config_ok(
    ctx: &Context<'_>,
    config: &Document,
    region: &Region,
) -> Result<bool, Error> {
    if let Some(mode) = config.get("mode") {
        if mode == &Null {
            ctx.say(format!(
                "Please set the mode for {} first in </config:1148650981555441897>!",
                region
            ))
            .await?;
            return Ok(false);
        }
    }

    if let Some(started) = config.get("tournament_started") {
        if started.as_bool().is_some() {
            ctx.say(format!("Tournament for {} has already started!", region))
                .await?;
            return Ok(false);
        }
    }

    // Handle other cases here if needed.

    Ok(true)
}
