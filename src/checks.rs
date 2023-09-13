use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use mongodb::Database;
use tracing::{info, instrument};


/// Checks if a tournament has started for the given region.
/// 
/// Make sure to pass in the database that corresponds to the region you want to check.
pub async fn check_if_tournament_started(database: Database) -> Result<bool, Error> {
    let config = database
        .collection::<Document>("Config")
        .find_one(None, None)
        .await?
        .unwrap();

    let tournament_started = config.get_bool("tournament_started").unwrap();

    Ok(tournament_started)
}

/// Checks if the user is a manager. Returns true if they are, false otherwise.
/// The bot owner may set new managers using the /set-manager command
/// 
/// Simply stick this at the top of your command to implement this check:
/// ```
/// if !user_is_manager(ctx).await? { return Ok(()) }
/// ```
#[instrument]
pub async fn user_is_manager(ctx: Context<'_>) -> Result<bool, Error> {
    info!("Checking permissions...");
    let guild_id = ctx.guild_id().unwrap();
    let user_id = ctx.author().id;
    let manager_doc_option = ctx
        .data()
        .database
        .general
        .collection::<Document>("Manager")
        .find_one(doc! {"guild_id": guild_id.clone().to_string()}, None)
        .await?;

    match manager_doc_option {
        Some(manager_doc) => {
            info!("Managers doc found");
            let manager_ids = manager_doc.get_array("manager_ids")?;
            let manager_id_strings = manager_ids
                .iter()
                .map(|id| id.to_string().strip_quote())
                .collect::<Vec<String>>();
            info!("Manager ids are: {:?}", manager_id_strings);

            let member_roles = guild_id.member(ctx, user_id).await?.roles;
            let member_role_id_strings = member_roles
                .iter()
                .map(|role| role.to_string())
                .collect::<Vec<String>>();
            info!(
                "The current member's roles are: {:?}",
                member_role_id_strings
            );

            for id in manager_id_strings {
                if member_role_id_strings.contains(&id) {
                    return Ok(true);
                }
            }
        }
        None => {
            ctx.say("No manager roles have been set for this server, you can set it by getting the bot owner to run /set-manager").await?;
            return Ok(false);
        }
    };

    ctx.send(|s| {
        s.ephemeral(true)
            .content("Sorry, you do not have the permissions required to run this command!")
            .reply(true)
    })
    .await?;

    return Ok(false);
}
