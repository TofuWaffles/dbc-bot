use crate::{bracket_tournament::region::Region, misc::CustomError, Context, Error};
use mongodb::{
    bson::{doc, Bson, Document},
    Collection,
};
use strum::IntoEnumIterator;

pub async fn find_player(ctx: Context<'_>) -> Result<Option<Document>, Error> {
    let mut player: Option<Document> = None;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Players");
        let filter = doc! {"discord_id": ctx.author().id.to_string()};
        match collection.find_one(filter, None).await {
            Ok(result) => match result {
                Some(p) => {
                    player = Some(p);
                    break;
                }
                None => continue,
            },
            Err(_) => {
                return Err(CustomError("Error finding player".to_string()).into());
            }
        }
    }
    Ok(player)
}
