use crate::bracket_tournament::config::{make_config, start_tournament_config};
use crate::bracket_tournament::mannequin::add_mannequin;
use crate::bracket_tournament::match_id::assign_match_id;
use crate::bracket_tournament::region::Region;
use crate::checks::{tournament_started, user_is_manager};
use crate::{Context, Error};
use mongodb::{bson, Database};
use mongodb::{
    bson::{doc, Bson, Document},
    options::AggregateOptions,
    Collection,
};
use poise::ReplyHandle;
use strum::IntoEnumIterator;
use tracing::{info, instrument};
const MINIMUM_PLAYERS: i32 = 3; // The minimum amount of players required to start a tournament

///Run this command once registration closes to start the tournament.
#[instrument]
#[poise::command(slash_command, guild_only, rename = "start-tournament")]
pub async fn start_tournament(
    ctx: Context<'_>,
    #[description = "(Optional) Start the tournament for specific region. By default, tournaments are started for all."]
    region_option: Option<Region>,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    info!("Attempting to start the tournament...");
    let msg = ctx
        .send(|s| {
            s.reply(true)
                .ephemeral(true)
                .content("Starting the tournament...")
        })
        .await?;

    let mut started_tournaments = Vec::<Region>::new();
    // Handling each region mathematical computations to preset brackets
    for region in Region::iter() {
        match region_option {
            Some(ref region_option) if region != *region_option => continue,
            _ => {}
        }

        info!("Starting tournament for region {}", region);
        let database = ctx.data().database.regional_databases.get(&region).unwrap();

        if !config_prerequisite(&ctx, &msg, database, &region).await?
            || !make_rounds(&ctx, &msg, database, &region).await?
        {
            continue;
        }

        prepare_round_1(&ctx, &msg, database, &region).await?;
        started_tournaments.push(region);
    }

    if started_tournaments.is_empty() {
        info!("No tournaments have been started");
        msg.edit(ctx, |m| {
            m.content("No tournaments have been started")
                .ephemeral(true)
        })
        .await?;
    } else {
        info!("Tournament(s) successfully started!");
        msg.edit(ctx, |m| {
            m.content(format!(
                "Tournament started for regions: {:#?}",
                started_tournaments
            ))
            .ephemeral(true)
        })
        .await?;
    }
    Ok(())
}

async fn config_prerequisite(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    database: &Database,
    region: &Region,
) -> Result<bool, Error> {
    let config_collection = database.collection("Config");
    let config: Document = match config_collection.find_one(None, None).await {
        Ok(None) => {
            let config = make_config();
            config_collection.insert_one(config.clone(), None).await?;
            return Ok(false);
        }
        Ok(Some(config)) => config,
        Err(_) => {
            msg.edit(*ctx, |s| s.content("Error occurred while finding config"))
                .await?;
            return Ok(false);
        }
    };
    if tournament_started(database).await? {
        msg.edit(*ctx, |s| s.content("Tournament is already started"))
            .await?;
        return Ok(false);
    }

    if let Some(mode) = config.get("mode") {
        match mode {
            Bson::String(_) => {}
            Bson::Null => {
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title(format!("Mode has not been set for {}", region))
                            .description(
                                "Please set the mode first at </set-config:1152203582356070450>",
                            )
                    })
                })
                .await?;
                return Ok(false);
            }
            _ => {
                // Handle other mode types if needed
            }
        }
    }
    Ok(true)
}

async fn make_rounds(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    database: &Database,
    region: &Region,
) -> Result<bool, Error> {
    let collection: Collection<Document> = database.collection("Players");
    let count = collection.count_documents(None, None).await? as i32;
    if count < MINIMUM_PLAYERS {
        msg.edit(*ctx, |s| {
            s.content(format!(
                "Not enough players to start a tournament at {}",
                region
            ))
        })
        .await?;
        return Ok(false);
    }
    let rounds = (count as f64).log2().ceil() as u32;
    let byes = 2_i32.pow(rounds) - count;
    info!(
        "Generating a bracket tournament with {} rounds and {} byes",
        rounds, byes
    );
    msg.edit(*ctx, |m| {
        m.content(format!("There are {} byes in region {}", byes, region))
    })
    .await?;
    match byes {
        0 => {}
        _ => {
            for _ in 1..=byes {
                let mannequin = add_mannequin(region, None, None);
                collection.insert_one(mannequin, None).await?;
            }
        }
    }
    info!("Writing round collections to the databases");
    for round in 1..=rounds {
        let collection_name = format!("Round {}", round);
        if !database
            .list_collection_names(None)
            .await?
            .contains(&collection_name)
        {
            database.create_collection(&collection_name, None).await?;
        }
    }

    let config = database.collection::<Document>("Config");
    config
        .update_one(doc! {}, start_tournament_config(&rounds), None)
        .await?; // Set total rounds, tournament_started to true and registration to false
    Ok(true)
}

async fn prepare_round_1(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    database: &Database,
    region: &Region,
) -> Result<(), Error> {
    let players: Collection<Document> = database.collection("Players");
    let pipeline = vec![
        bson::doc! { "$match": bson::Document::new() },
        bson::doc! { "$out": "Round 1" },
    ];

    let options = AggregateOptions::builder().allow_disk_use(true).build();
    players.aggregate(pipeline, Some(options)).await?;

    assign_match_id(region, database).await?;
    msg.edit(*ctx, |s| {
        s.content(format!("Complete set up the tournament for {}", region))
    })
    .await?;
    Ok(())
}
