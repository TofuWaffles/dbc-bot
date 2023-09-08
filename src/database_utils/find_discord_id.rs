use crate::{bracket_tournament::{region::Region, config::get_config}, misc::QuoteStripper, Context};
use mongodb::{
    bson::{doc, Document, Bson},
    Collection,
};
use strum::IntoEnumIterator;
use tracing::{error, info, instrument};

/// Checks if a player with the given Discord user ID exists in the database.
///
/// This function queries the specified MongoDB collection for a document that matches
/// the Discord user ID of the invoking user in the provided `Context`. If a matching
/// document is found, it returns the player's data as a `Document`. If no matching
/// document is found or an error occurs during the query, it returns `None`.
///
/// # Parameters
///
/// - `ctx`: A reference to the Serenity `Context` containing information about the
///          current Discord interaction and server context.
/// - `discord_id`: An optional `String` containing the Discord user ID of the player. If this is not given, Discord ID will be retrieve
/// from `ctx.author().id`
/// # Returns
///
/// An `Option<Document>` representing the player's data if found, or `None` if the player
/// is not in the database or an error occurs.
///
/// # Examples
///
/// ```rust
/// use serenity::prelude::*;
/// use mongodb::Collection;
/// use mongodb::bson::doc;
///
/// struct PlayerDataKey;
///
/// impl TypeMapKey for PlayerDataKey {
///     type Value = Collection<Document>;
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let ctx = Context::new().await; // Create a Serenity context (replace with your actual setup).
///
///     match is_in_db(&ctx).await {
///         Some(player) => {
///             println!("The player exists in the database. Player data: {:?}", player);
///         }
///         None => {
///             println!("The player does not exist in the database.");
///         }
///     }
/// }
/// ```
#[instrument]
pub async fn find_discord_id(ctx: &Context<'_>, discord_id: Option<String>) -> Option<Document> {
    let invoker_id = match discord_id {
        Some(id) => id,
        None => ctx.author().id.to_string(),
    };
    // Define a variable to hold the result
    let mut result: Option<Document> = None;

    info!("Iterating through and checking the database for each region");
    // Iterate through the regions and check each database
    for region in Region::iter() {
        info!("Checking database for region: {}", region);
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let config = get_config(database).await;
        let round = match config.get("round") {
            Some(round) => {
                if let Bson::Int32(0) = round {
                    format!("Player")
                } else {
                    format!("Round {}", round.as_i32().unwrap())
                }
            },
            None => {
                error!("Error while getting round from config");
                return None;
            }
        };

        let player_data: Collection<Document> = database.collection(round.as_str());
        match player_data
            .find_one(doc! { "discord_id": &invoker_id.strip_quote()}, None)
            .await
        {
            Ok(Some(player)) => {
                result = Some(player);
                break; // Exit the loop when a match is found
            }
            Ok(None) => {
                continue;
            }
            Err(err) => {
                error!("Error while querying database: {:?}", err);
                result = None;
                break;
            }
        }
    }
    result
}
