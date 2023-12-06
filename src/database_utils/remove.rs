use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

pub async fn remove_player(ctx: &Context<'_>, player: &Document) -> Result<(), Error> {
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection = database.collection::<Document>("Players");
    collection
        .delete_one(doc! {"_id": player.get("_id")}, None)
        .await?;
    Ok(())
}
