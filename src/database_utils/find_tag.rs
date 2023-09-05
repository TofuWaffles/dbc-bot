use crate::{
  Context,
  bracket_tournament::region::Region
};
use mongodb::{
  bson::{
      doc,
      Document
  },
  Collection,
};
use strum::IntoEnumIterator;
use tracing::{error, info, instrument};

/// Asynchronously searches for a player's tag in the regional databases.
///
/// # Arguments
///
/// * `ctx` - The context of the application.
/// * `tag` - The tag to search for.
///
/// # Returns
///
/// An `Option<Document>` representing the player's data if found, or `None` if not found or an error occurred.
///
/// # Example
///
/// ```rust
/// let player_data = find_tag(&ctx, "player123".to_string()).await;
/// match player_data {
///     Some(player) => {
///         println!("Player found: {:?}", player);
///     }
///     None => {
///         println!("Player not found.");
///     }
/// }
/// ```
#[instrument]
pub async fn find_tag(ctx: &Context<'_>, tag: &String) -> Option<Document> {

  let mut result: Option<Document> = None;
  info!("Iterating through and checking the database for each region");
  let proper_tag = match tag.starts_with('#') {
    true => &tag[1..],
    false => tag,
  };
  for region in Region::iter() {
      info!("Checking database for region: {}", region);

      let database = ctx.data().database.regional_databases.get(&region).unwrap();
      let player_data: Collection<Document> = database.collection(format!("Player").as_str());

      match player_data
          .find_one(doc! { "tag": format!("#{}",&proper_tag)}, None)
          .await
      {
          Ok(Some(player)) => {
              result = Some(player);
              break;
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
  // Return the result
  result
}
