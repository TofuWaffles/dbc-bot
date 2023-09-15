use crate::misc::QuoteStripper;
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::Database;
use tracing::{info, instrument};

/// Check if a tournament has started.
///
/// This asynchronous function queries the database to determine whether a tournament
/// has started based on the value of the "tournament_started" field in the "Config" collection.
///
/// # Arguments
///
/// * `database` - A reference to the database connection (provided by the `Database` type).
///
/// # Returns
///
/// If the function successfully retrieves the "tournament_started" field from the database,
/// it returns a `Result` containing a boolean value indicating whether the tournament has started.
///
/// If an error occurs during the database query or while extracting the value,
/// an `Err` variant of the `Result` containing an `Error` type is returned.
///
/// # Examples
///
/// ```
/// use your_module::tournament_started;
/// use your_module::Database;
///
/// #[tokio::main]
/// async fn main() {
///     let database = Database::connect().await.expect("Failed to connect to the database");
///
///     match tournament_started(&database).await {
///         Ok(has_started) => {
///             if has_started {
///                 println!("The tournament has started.");
///             } else {
///                 println!("The tournament has not started yet.");
///             }
///         },
///         Err(err) => {
///             eprintln!("Error checking tournament status: {:?}", err);
///         },
///     }
/// }
/// ```
///
/// This function is designed to be used in an asynchronous context, typically with the Tokio runtime.
///
/// Make sure to handle potential errors that may occur during the database query.
///
/// # Note
///
/// The function assumes that the "Config" collection in the database contains a boolean field
/// named "tournament_started" to indicate the status of the tournament.
///
/// Ensure that your database connection is properly established before calling this function.
pub async fn tournament_started(database: &Database) -> Result<bool, Error> {
    let config = database
        .collection::<Document>("Config")
        .find_one(None, None)
        .await?
        .unwrap();

    let tournament_started = config.get_bool("tournament_started")?;

    Ok(tournament_started)
}

/// Check if a user is a manager in a specific guild.
///
/// This asynchronous function is used to determine whether a user has manager permissions
/// within a particular guild. Manager permissions are typically associated with a specific role.
///
/// # Arguments
///
/// * `ctx` - A reference to the context (of type `Context`) containing information about
///   the user, the guild, and the database connection.
///
/// # Returns
///
/// If the user has manager permissions (as determined by having a specific role),
/// the function returns `Ok(true)`. If the user does not have the required permissions,
/// it returns `Ok(false)`. If an error occurs during the permission check or while sending
/// a response message, an `Err` variant containing an `Error` type is returned.
///
/// # Example
///
/// ```
/// use crate::checks::{user_is_manager, Context};
/// use crate::Error;
///
/// #[tokio::main]
/// async fn main() {
///     let ctx = Context::new(/* initialize your context here */);
///
///     match user_is_manager(ctx).await {
///         Ok(has_manager_permissions) => {
///             if has_manager_permissions {
///                 println!("User has manager permissions.");
///             } else {
///                 println!("User does not have manager permissions.");
///             }
///         },
///         Err(err) => {
///             eprintln!("Error checking user permissions: {:?}", err);
///         },
///     }
/// }
/// ```
///
/// This function assumes that user permissions are determined based on the presence
/// of a specific role associated with manager permissions in the guild.
///
/// Make sure to handle potential errors that may occur during the role check or when
/// sending a response message.
#[instrument]
pub async fn user_is_manager(ctx: Context<'_>) -> Result<bool, Error> {
    info!("Checking permissions...");
    let guild_id = ctx.guild_id().unwrap().to_string();
    let database = &ctx.data().database.general;

    let mut managers = database
        .collection::<Document>("Managers")
        .find(doc! {"guild_id": &guild_id}, None)
        .await?;
    while let Some(manager) = managers.try_next().await? {
        let role_id = manager.get("role_id").unwrap().to_string().strip_quote();
        if ctx
            .author()
            .has_role(
                ctx.http(),
                guild_id.parse::<u64>().unwrap(),
                role_id.parse::<u64>().unwrap(),
            )
            .await?
        {
            return Ok(true);
        }
    }
    ctx.send(|s| {
        s.content("Sorry, you do not have the permissions required to run this command!")
            .ephemeral(true)
            .reply(true)
    })
    .await?;
    Ok(false)
}
