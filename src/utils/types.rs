/*
This file contains various common type definitions or type aliases that are used throughout the bot.
For now, these are mainly used for type annotations
*/

pub struct Data {}
pub type Error = Box<dyn std::error::Error + Send + Sync>;  // Error type used by Poise
pub type Context<'a> = poise::Context<'a, Data, Error>;     // Context type as defined by poise
