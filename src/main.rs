/*
TODO:
- Build out self role and new channel alert feature
- Plan and subsequently build the tournament bracket feature
*/
mod bracket_tournament;
mod commands;
mod utils;

use poise::serenity_prelude as serenity;


// This data struct is used to pass data (such as the db_pool) to the context object
pub struct Data {
    // db_pool: sqlx::PgPool,
    // self_role_messages: DashMap<i64, self_role::SelfRoleMessage>, // Required for the self_role module
}
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() {
    // Load the environment variable from the .env file
    dotenv::dotenv().expect("Unable to load the .env file. Check if it has been created.");

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set. Set it as an environment variable.");

    // A list of commands to register. Remember to add the function for the command in this vec, otherwise it won't appear in the command list.
    // Might be better to find a more scalable and flexible solution down the line.
    let commands = vec![commands::ping::ping(), commands::player::player(), commands::battle_log::latest_log()];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        });

    println!("The bot is starting...");
    framework.run().await.unwrap();
}