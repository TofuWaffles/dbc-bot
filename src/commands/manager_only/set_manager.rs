use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use mongodb::Database;
use poise::serenity_prelude::Role;
use tracing::{info, instrument};

/// Set a role as a manager to access manager-only commands. Only the bot owner can run this.
#[instrument]
#[poise::command(slash_command, guild_only, owners_only, rename = "set-manager")]
pub async fn set_manager(ctx: Context<'_>, role: Role) -> Result<(), Error> {
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
