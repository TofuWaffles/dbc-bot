/*
TODO:
- Build out self role and new channel alert feature
- Plan and subsequently build the tournament bracket feature
*/
mod bracket_tournament;
mod commands;
mod misc;
mod self_role;

use dashmap::DashMap;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Database,
};

use crate::misc::Region;
use futures::stream::TryStreamExt;
use poise::{
    serenity_prelude::{self as serenity, GatewayIntents},
    Event, FrameworkError,
};
use std::collections::HashMap;
use std::fs::File;
use std::sync::Arc;
use tracing::{error, info, instrument, trace};
use tracing_subscriber::{filter, prelude::*};

// Rest of your code here
#[derive(Debug)]
struct Databases {
    registration: Database,
    regional_databases: HashMap<Region, Database>,
}
// This data struct is used to pass data (such as the db_pool) to the context object
#[derive(Debug)]
pub struct Data {
    database: Databases,
    self_role_messages: DashMap<i64, self_role::SelfRoleMessage>, // Required for the self_role module
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
        commands::ping::ping(),
        commands::player::player(),
        commands::battle_log::latest_log(),
        commands::register::register(),
        commands::create_self_role_message::create_self_role_message(),
        // commands::submit::submit(),
        commands::db_handler::get_individual_player_data(),
        commands::db_handler::get_all_players_data(),
        commands::deregister::deregister(),
        commands::starttournament::start_tournament(),
        commands::club::club(),
        commands::region_proportion::region_proportion(),
        commands::reset_match_id::reset_match_id(),
        commands::fill_manequins::fill_mannequins(),
    ];

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set. Set it as an environment variable.");

    info!("Setting up the bot...");

    info!("Generating options");
    let options = poise::FrameworkOptions {
        commands,
        event_handler: |ctx, event, _framework, data| {
            Box::pin(async move {
                match event {
                    Event::Ready { data_about_bot } => {
                        let bot_name = data_about_bot.user.name.to_owned();
                        info!("{username} is online", username = bot_name);
                        println!("{} is online!", bot_name);
                    }

                    Event::InteractionCreate { interaction } => match interaction {
                        serenity::Interaction::MessageComponent(message_component_interaction) => {
                            match message_component_interaction.data.component_type {
                                // We exhaustively check the specific interaction type so that we don't have to do it inside every function
                                serenity::ComponentType::Button => {
                                    self_role::handle_button::handle_selfrole_button(
                                        message_component_interaction,
                                        ctx,
                                        data,
                                    )
                                    .await?;
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    },
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
                let mut self_role_data = database
                    .registration
                    .collection::<self_role::SelfRoleMessage>("SelfRoleMessage")
                    .find(None, None)
                    .await
                    .expect("Self Role data not found.");
                let self_role_messages = DashMap::<i64, self_role::SelfRoleMessage>::new();
                while let Some(self_role_individual_data) = self_role_data.try_next().await? {
                    self_role_messages.insert(
                        self_role_individual_data.message_id,
                        self_role::SelfRoleMessage {
                            message_id: self_role_individual_data.message_id,
                            guild_id: self_role_individual_data.guild_id,
                            role_id: self_role_individual_data.role_id,
                            ping_channel_id: self_role_individual_data.ping_channel_id,
                        },
                    );
                }
                Ok(Data {
                    database,
                    self_role_messages,
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
    trace!("Preparing database...");

    let db_uri = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set. Set it as an environment variable.");

    let options =
        ClientOptions::parse_with_resolver_config(&db_uri, ResolverConfig::cloudflare()).await?;

    let client = Client::with_options(options)?;
    let registration = client.database("SelfRole");
    let mut regional_database: HashMap<Region, Database> = HashMap::new();
    regional_database.insert(Region::APAC, client.database("APAC"));
    regional_database.insert(Region::EU, client.database("EU"));
    regional_database.insert(Region::NASA, client.database("NASA"));
    let required_collections = vec!["Player", "SelfRoleMessage"];

    // We want to preload some of these collections, which is why we create this collection if it does not exist
    // Errors if the DB already exists and skips creation
    for collection in required_collections {
        registration
            .create_collection(collection, None)
            .await
            .unwrap_or_else(|e| info!("{:?}", e));
    }

    info!("Database prepared successfully!");

    Ok(Databases {
        registration: registration,
        regional_databases: regional_database,
    })
}

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
                "Error executing the pre-command check for {} in guild {}: {:?}",
                ctx.command().qualified_name,
                ctx.guild().unwrap().name,
                error
            );
        }
        FrameworkError::ArgumentParse { error, input, ctx } => {
            error!(
                "Error parsing arguments for {} in guild {} with input(s) {:?}: {:?}",
                ctx.command().qualified_name,
                ctx.guild().unwrap().name,
                input,
                error
            );
        }
        _ => {
            error!("An unknown error occurred");
        }
    }
}
