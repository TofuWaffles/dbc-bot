use std::{collections::HashMap, fs::File, str::FromStr, sync::Arc};
use strum::IntoEnumIterator;
use mongodb::{
    bson::{Document, doc},
    options::{ClientOptions, ResolverConfig},
    Client, Collection, Database,
};
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents, UserId},
    Event, FrameworkError,
};
use tracing::{error, info, instrument, trace};
use tracing_subscriber::{filter, prelude::*};

mod bracket_tournament;
mod checks;
mod commands;
mod database_utils;
mod misc;

use crate::bracket_tournament::region::Region;


#[derive(Debug)]
struct Databases {
    general: Database,
    regional_databases: HashMap<Region, Database>,
}
// This data struct is used to pass data (such as the db_pool) to the context object
#[derive(Debug)]
pub struct Data {
    database: Databases,
    // managers: Vec<u64>
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variable from the .env file
    dotenv::dotenv().expect("Unable to load the .env file. Check if it has been created.");

    if let Err(e) = create_subscriber() {
        // Change to a panic!() if you really need logging to work
        println!("Unable to create subscriber: {}", e);
    }

    if let Err(e) = run().await {
        panic!("Error trying to run the bot: {}", e);
    }
}

#[instrument]
async fn run() -> Result<(), Error> {
    // A list of commands to register. Remember to add the function for the command in this vec, otherwise it won't appear in the command list.
    // Might be better to find a more scalable and flexible solution down the line.
    let commands = vec![
        // commands::battle_log::latest_log(),
        commands::draco::draco(),
        // commands::ping::ping(),
        // commands::player::player(),
        commands::register::register(),
        commands::register::deregister(),
        commands::submit::submit(),
        commands::view::view_managers(),
        commands::view::view_opponent(),

        commands::manager_only::db_handler::get_individual_player_data(),
        commands::manager_only::db_handler::get_all_players_data(),
        commands::manager_only::config::config(),
        commands::manager_only::start_tournament::start_tournament(),
        commands::manager_only::region_proportion::region_proportion(),
        commands::manager_only::reset::reset(),
        commands::manager_only::fill_manequins::fill_mannequins(),
        commands::manager_only::set_round::set_round(),
        commands::manager_only::disqualify::disqualify(),
        commands::manager_only::set_manager::set_manager(),
    ];

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set. Set it as an environment variable.");
    let owner_id =
        std::env::var("OWNER_ID").expect("OWNER_ID is not set. Set it as an environment variable.");

    info!("Setting up the bot...");

    info!("Generating options");
    let options = poise::FrameworkOptions {
        commands,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                match event {
                    Event::Ready { data_about_bot } => {
                        let bot_name = data_about_bot.user.name.to_owned();
                        info!("{username} is online", username = bot_name);
                        println!("{} is online!", bot_name);
                    }

                    _ => (),
                }

                Ok(())
            })
        },
        pre_command: |ctx| {
            Box::pin(async move {
                trace!(
                    "Executing command: {cmd_name}",
                    cmd_name = ctx.command().qualified_name
                );
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                trace!(
                    "Finished executing command: {cmd_name}",
                    cmd_name = ctx.command().qualified_name
                );
            })
        },
        owners: std::collections::HashSet::from([UserId::from_str(&owner_id)?]),
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };
    info!("Options generated successfully!");

    let database = prepare_databases().await?;

    info!("Generating framework...");
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    database,
                    // managers
                })
            })
        })
        .initialize_owners(true)
        .options(options)
        .intents(
            GatewayIntents::non_privileged()
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::GUILD_MEMBERS,
        )
        .build()
        .await?;
    info!("Framework generated successfully!");

    let shard_manager = framework.shard_manager().clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register the ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    info!("Bot starting...");
    println!("Starting the bot...");
    framework.start().await?;

    Ok(())
}

#[instrument]
async fn prepare_databases() -> Result<Databases, Error> {
    info!("Preparing database...");

    let db_uri = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set. Set it as an environment variable.");

    let options =
        ClientOptions::parse_with_resolver_config(&db_uri, ResolverConfig::cloudflare()).await?;

    let client = Client::with_options(options)?;
    let general = client.database("General");

    let mut regional_database: HashMap<Region, Database> = HashMap::new();
    regional_database.insert(Region::APAC, client.database("APAC"));
    regional_database.insert(Region::EU, client.database("EU"));
    regional_database.insert(Region::NASA, client.database("NASA"));
    let required_collections = vec!["Players", "Managers"];
    let required_regional_collections = bracket_tournament::config::make_config();

    // We want to preload some of these collections, which is why we create this collection if it does not exist
    // Errors if the collection already exists and skips creation
    for collection in required_collections {
        general
            .create_collection(collection, None)
            .await
            .unwrap_or_else(|e| info!("{:?}", e));
    }

    for region in Region::iter() {
        let database = regional_database.get(&region).unwrap();
        let collection_names = database.list_collection_names(None).await?;
        if !collection_names.iter().any(|s| s == "Config") {
            database.create_collection("Config", None).await?;
            let collection = database.collection("Config");
            collection
                .insert_one(required_regional_collections.clone(), None)
                .await?;
            info!("Config collection created for {}", region);
        } else {
            let collection: Collection<Document> = database.collection("Config");
            if collection.count_documents(None, None).await? == 0 {
                collection
                    .insert_one(required_regional_collections.clone(), None)
                    .await?;
                info!(
                    "Config document is created successfully in the database of {}",
                    region
                );
            }
            info!("Config already exists in {}", region);
        }
        database
            .create_collection("Players", None)
            .await
            .unwrap_or_else(|e| info!("{:?}", e));
    }

    info!("Databases prepared successfully!");

    Ok(Databases {
        general,
        regional_databases: regional_database,
    })
}
// async fn retrieve_managers(database: &Database) -> Vec<u64>{
//     let mut managers_list = vec![];
//     let mut managers = database
//         .collection::<Document>("Managers")
//         .find(doc! {"guild_id": &guild_id}, None)
//         .await;
//     while let Some(manager) = managers.try_next().await{
//         managers_list.push(manager.get("role_id").unwrap().to_string().parse::<u64>().unwrap());
//     }
//     managers_list
// }
// Create the subscriber to listen to logging events
fn create_subscriber() -> Result<(), Error> {
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let file = File::create("debug.log")?;
    let debug_log = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));

    tracing_subscriber::registry()
        .with(
            stdout_log
                .with_filter(filter::LevelFilter::INFO)
                .and_then(debug_log),
        )
        .init();

    Ok(())
}

async fn on_error(error: FrameworkError<'_, Data, Error>) {
    match error {
        FrameworkError::Setup { error, .. } => {
            panic!("Failed to start the bot: {:?}", error);
        }
        FrameworkError::Command { error, ctx } => {
            error!(
                "Error executing command {} in guild: {}: {:?}",
                ctx.command().qualified_name,
                ctx.guild().unwrap().name,
                error
            );
        }
        FrameworkError::CommandCheckFailed { error, ctx } => {
            error!(
                "Error executing the pre-command check for {}{:?}",
                ctx.command().qualified_name,
                error
            );
        }
        FrameworkError::ArgumentParse { error, input, ctx } => {
            error!(
                "Error parsing arguments for {} with input(s) {:?}: {:?}",
                ctx.command().qualified_name,
                input,
                error
            );
        }
        FrameworkError::EventHandler {
            error,
            ctx: _,
            event: _,
            framework: _,
        } => {
            error!("Error executing event handler: {:?}", error);
        }
        FrameworkError::CommandPanic { payload, ctx } => {
            error!(
                "A command has panicked: {:?}",
                payload.unwrap_or_else(|| "Failed to get panic message.".to_string()),
            );
            ctx.say(
                "The command has failed, please contact the bot operators if the issue persists.",
            )
            .await
            .unwrap();
        }
        FrameworkError::CommandStructureMismatch {
            description,
            ctx: _,
        } => {
            error!(
                "The command's structure had a mismatch: {}.\n Most likely the command was updated but not reregistered on Discord. Try reregistering the command and try again",
                description
            );
        }
        FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
        } => {
            ctx.say(format!(
                "This command is still on cooldown. Try again in {} seconds",
                remaining_cooldown.as_secs()
            ))
            .await
            .unwrap();
        }
        FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
        } => {
            error!(
                "The bot lacks the following permissions to complete the task: {:?}",
                missing_permissions
            );
            ctx.say("I do not have the required permissions to do this, sorry :(")
                .await
                .unwrap();
        }
        FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
        } => {
            info!(
                "The user requires the following permissions to complete the task: {:?}",
                missing_permissions
            );
            ctx.say("You do not have the required permissions to do this, sorry :(")
                .await
                .unwrap();
        }
        FrameworkError::NotAnOwner { ctx } => {
            ctx.say("This command is only available to the bot owners.")
                .await
                .unwrap();
        }
        FrameworkError::GuildOnly { ctx } => {
            ctx.say("This command is only available in a Discord server.")
                .await
                .unwrap();
        }
        FrameworkError::DmOnly { ctx } => {
            ctx.say("This command is only available in a DM.")
                .await
                .unwrap();
        }
        FrameworkError::NsfwOnly { ctx } => {
            ctx.say("This command is only available in a NSFW channel.")
                .await
                .unwrap();
        }
        FrameworkError::DynamicPrefix { error, ctx: _, msg } => {
            error!(
                "A dynamic prefix error occured for message \"{:?}\": {:?}",
                msg, error
            );
        }
        FrameworkError::UnknownCommand {
            ctx: _,
            msg: _,
            prefix,
            msg_content,
            framework: _,
            invocation_data: _,
            trigger: _,
        } => {
            info!("A user tried to trigger an unknown command with the bot's prefix of {} with the message: {}", prefix, msg_content);
        }
        FrameworkError::UnknownInteraction {
            ctx: _,
            framework: _,
            interaction,
        } => {
            error!(
                "An interaction occured that was unspecified: {:?}",
                interaction
            );
        }
        _ => {
            error!("An unknown error occurred!");
        }
    }
}
