/*
TODO:
- Build out self role and new channel alert feature
- Plan and subsequently build the tournament bracket feature
*/
mod bracket_tournament;
mod commands;
mod self_role;
mod utils;

use dashmap::DashMap;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::doc,
    options::FindOptions,
    options::{ClientOptions, ResolverConfig},
    Client,
};
use poise::serenity_prelude::{
    self as serenity, GatewayIntents, InteractionType, MessageComponentInteraction,
};

// This data struct is used to pass data (such as the db_pool) to the context object
pub struct Data {
    db_client: mongodb::Client,
    self_role_messages: DashMap<i64, self_role::SelfRoleMessage>, // Required for the self_role module
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variable from the .env file
    dotenv::dotenv().expect("Unable to load the .env file. Check if it has been created.");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set. Set it as an environment variable.");

    let db_uri = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set. Set it as an environment variable.");

    // A list of commands to register. Remember to add the function for the command in this vec, otherwise it won't appear in the command list.
    // Might be better to find a more scalable and flexible solution down the line.
    let commands = vec![
        commands::ping::ping(),
        commands::player::player(),
        commands::register::register(),
        commands::battle_log::log(),
        commands::db_handler::get_player_data(),
    ];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let options = ClientOptions::parse_with_resolver_config(
                    &db_uri,
                    ResolverConfig::cloudflare(),
                )
                .await?;
                let db_client = Client::with_options(options)?;
                let mut self_role_data = db_client
                    .database("DBC-bot")
                    .collection::<self_role::SelfRoleMessage>("SelfRoles")
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
                    db_client,
                    self_role_messages,
                })
            })
        });

    println!("The bot is starting...");
    framework.run().await.unwrap();
}
