/*
TODO:
- Build out self role and new channel alert feature
- Plan and subsequently build the tournament bracket feature
*/

mod ping;

use poise::serenity_prelude::{
    self as serenity,
    GatewayIntents,
};

// This data struct is used to pass data (such as the db_pool) to the context object
pub struct Data {
    db_pool: sqlx::PgPool,
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

async fn run() -> Result<(), Error> {
    let options = poise::FrameworkOptions {
        commands: vec![ping::ping()],
        ..Default::default()
    };

    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(
            &std::env::var("DATABASE_URL").expect("Missing DATABASE_URL environment variable."),
        )
        .await?;

    let framework = poise::Framework::builder()
        .token(
            std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN environment variable."),
        )
        .setup(move |ctx, _ready, _framework| {
            Box::pin(async move {
                ctx.set_activity(serenity::Activity::playing("Discord Brawl Cup"))
                    .await;

                Ok(Data { db_pool })
            })
        })
        .initialize_owners(true)
        .options(options)
        .intents(GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .build()
        .await?;

    let shard_manager = framework.shard_manager().clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register the ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    println!("Starting the bot...");
    framework.start().await?;

    Ok(())
}
