use crate::misc::QuoteStripper;
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use tracing::{info, instrument};

/// Checks if a tournament has started based on the provided configuration document.
///
/// # Arguments
///
/// * `config` - A reference to a BSON `Document` containing configuration data.
///
/// # Returns
///
/// Returns a `Result<bool, Error>`. If the "tournament_started" key is found in the
/// `config` document and its value is a boolean, it returns `Ok(true)` or `Ok(false)`
/// depending on the boolean value. If the key is not found or the value is not a boolean,
/// it returns an `Err` containing an error description.
///
/// # Example
///
/// ```
/// let config = doc! {
///     "tournament_started": Bson::Boolean(true), // Replace with your actual configuration data.
/// };
///
///     // Call the tournament_started function.
/// match tournament_started(&config) {
///     Ok(started) => {
///         if started {
///             println!("The tournament has started.");
///         } else {
///             println!("The tournament has not started.");
///         }
///     }
///     Err(err) => {
///         eprintln!("Error: {}", err);
///     }
/// }
/// ```
/// Ensure that your database connection is properly established before calling this function.
pub async fn tournament_started(config: &Document) -> Result<bool, Error> {
    println!("tournament_started function runs");
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
