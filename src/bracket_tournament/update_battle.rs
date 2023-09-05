use crate::Error;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
/// Asynchronously updates battle-related information in a MongoDB database.
///
/// This function updates the "battle" field to `true` for all documents in the collection
/// associated with the specified `round` that match the given `match_id`.
///
/// # Arguments
///
/// * `database` - A reference to the MongoDB `Database` where the data should be updated.
/// * `round` - An integer representing the round for which the battle information should be updated.
/// * `match_id` - An integer representing the unique identifier of the match for which the battle information should be updated.
///
/// # Returns
///
/// A `Result` indicating success (`Ok(())`) or failure (`Err(Error)`).
///
/// # Errors
///
/// This function can return various errors, including but not limited to:
/// - Database connection errors.
/// - MongoDB update errors.
/// - Permissions issues.
///
/// # Example
///
/// ```rust
/// use mongodb::Database;
/// use bson::doc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let database: Database = get_database_connection().await?;
///     let round = 2;
///     let match_id = 12345;
///
///     match update_battle(&database, round, match_id).await {
///         Ok(_) => {
///             println!("Battle information updated successfully.");
///         }
///         Err(err) => {
///             eprintln!("Error updating battle information: {}", err);
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn update_battle(database: &Database, round: i32, match_id: i32) -> Result<(), Error> {
    let current_round: Collection<Document> =
        database.collection(format!("Round {}", round).as_str());
    let filter = doc! {
        "match_id": match_id
    };
    let update = doc! {
        "$set": {
           "battle": true
        }
    };
    current_round.update_many(filter, update, None).await?;
    println!("Battle is updated!");

    Ok(())
}
