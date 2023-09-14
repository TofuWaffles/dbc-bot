use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::Role;
use tracing::{info, instrument};

/// Set a role as a manager to access manager-only commands. Only the bot owner can run this.
#[instrument]
#[poise::command(slash_command, guild_only, owners_only, rename = "set-manager")]
pub async fn set_manager(ctx: Context<'_>, role: Role) -> Result<(), Error> {
    info!("Setting manager for {}", role);
    let database = &ctx.data().database.general;
    let guild_id = ctx.guild_id().unwrap().to_string();
    let role_id_string = role.id.to_string();

    // We want to check if a document exists for this guild, if not, we create one
    let managers_doc = match database
        .collection::<Document>("Manager")
        .find_one(doc! {"guild_id": guild_id.clone()}, None)
        .await?
    {
        Some(doc) => {
            info!(
                "Found existing document for the current guild: {}",
                guild_id
            );
            doc
        }
        None => {
            info!(
                "No document found for the current guild: {}.\n Creating one...",
                guild_id
            );
            database
                .collection::<Document>("Manager")
                .insert_one(
                    doc! {"guild_id": guild_id, "manager_ids": vec![role_id_string]},
                    None,
                )
                .await?;
            info!("Role saved as manager");
            ctx.say("Successfully save the role as a manager!").await?;
            return Ok(());
        }
    };

    // Then, we check if the current role has already been added as a manager in this guild
    let managers_vec = managers_doc
        .get_array("manager_ids")?
        .iter()
        .map(|x| x.as_str().unwrap_or("0").to_string().strip_quote())
        .collect::<Vec<String>>();
    info!("The current managers are: {:?}", managers_vec);

    let manager_dupe = managers_vec.contains(&role_id_string);

    if manager_dupe {
        info!("Found dupe for this id");
        ctx.say("This role is already a manager").await?;
        return Ok(());
    }

    info!("No dupe manager found. Continuing to insert new manager");
    let doc_id = managers_doc.get_object_id("_id")?;
    let updated_managers_result = database
        .collection::<Document>("Manager")
        .update_one(
            doc! {"_id": doc_id},
            doc! {"$push": {"manager_ids": role_id_string}},
            None,
        )
        .await?;
    info!("Update managers result: {:?}", updated_managers_result);
    ctx.say("Successfully saved the role as a manager").await?;

    Ok(())
}
