use crate::{bracket_tournament::region::Region, Context};
use mongodb::{
    bson::{doc, Bson, Document},
    Collection,
};
use tracing::{error, info, instrument};

#[instrument]
pub async fn find_enemy(
    ctx: &Context<'_>,
    region: &Region,
    round: &i32,
    match_id: &i32,
    other_tag: &str,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection(format!("Round {}", round).as_str());
    info!("Searching opponent for match {}", match_id);
    let filter = doc! {
        "match_id": match_id,
        "tag": {
           "$ne": other_tag
        }
    };
    info!("Using filter: {:?}", filter);
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => Some(enemy),
        Ok(None) => None,
        Err(err) => {
            info!("Error while querying database: {:?}", err);
            None
        }
    }
}

pub fn is_mannequin(enemy: &Document) -> bool {
    enemy.get("tag").unwrap() == &Bson::Null
}
