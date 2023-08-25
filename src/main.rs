/*
TODO:
- Build out self role and new channel alert feature
- Plan and subsequently build the tournament bracket feature
*/

mod ping;
mod self_role;

use dashmap::DashMap;
use poise::serenity_prelude::InteractionType;
use poise::serenity_prelude::{self as serenity, GatewayIntents, MessageComponentInteraction};
use poise::Event;
use tracing::{Level, span, event, instrument, info};

// This data struct is used to pass data (such as the db_pool) to the context object
pub struct Data {
    db_pool: sqlx::PgPool,
    self_role_messages: DashMap<i64, self_role::SelfRoleMessage>, // Required for the self_role module
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variable from the .env file
    dotenv::dotenv().expect("Unable to load the .env file. Check if it has been created.");

    if let Err(e) = run().await {
        println!("Error trying to run the bot: {}", e);
    }
}

#[instrument]
async fn run() -> Result<(), Error> {
    info!("Setting up the bot...");

    // The list of commands goes here
    info!("Generating options");
    let options = poise::FrameworkOptions {
        commands: vec![ping::ping()],
        event_handler: |ctx, event, _framework, data| {
            Box::pin(async move {
                match event {
                    Event::Ready { data_about_bot } => {
                        println!("{} is connected!", data_about_bot.user.name);
                    }
                    Event::InteractionCreate { interaction } => match interaction {
                        serenity::Interaction::MessageComponent(message_component_interaction) => {
                            match message_component_interaction.data.component_type {
                                // We exhaustively check the specific interaction type so that we don't have to do it inside every function
                                serenity::ComponentType::Button => self_role::handle_button::handle_selfrole_button(message_component_interaction, ctx, data).await?,
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
        ..Default::default()
    };
    info!("Options generated successfully!");

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(
            &std::env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable."),
        )
        .await?;

    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // We want to load the messages into a hashmap for quick lookup in the self_role event handler
    let self_role_messages = DashMap::<i64, self_role::SelfRoleMessage>::new();

    for msg in sqlx::query_as!(
        self_role::SelfRoleMessage,
        "SELECT message_id, guild_id, role_id, ping_channel_id FROM self_role_message;"
    )
    .fetch_all(&db_pool)
    .await?
    {
        self_role_messages.insert(msg.message_id, msg);
    }

    info!("Generating framework...");
    let framework = poise::Framework::builder()
        .token(std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable."))
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                ctx.set_activity(serenity::Activity::playing("Discord Brawl Cup"))
                    .await;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {
                    db_pool,
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
