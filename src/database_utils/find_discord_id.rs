use crate::bracket_tournament::{config::get_config, region::Region};
use crate::misc::QuoteStripper;
use crate::Context;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;

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
pub async fn find_discord_id(ctx: &Context<'_>, discord_id: Option<String>) -> Option<Document> {
    let invoker_id = match discord_id {
        Some(id) => id,
        None => ctx.author().id.to_string(),
    };
    // Define a variable to hold the result
    let mut result: Option<Document> = None;

    // Iterate through the regions and check each database
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let player_data: Collection<Document> = database.collection(format!("Player").as_str());
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
                eprintln!("Error while querying database: {:?}", err);
                result = None;
                break;
            }
        }
    }
    result
}
