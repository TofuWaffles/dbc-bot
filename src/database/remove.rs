use super::config::get_config;
use super::mannequin::add_mannequin;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

pub async fn remove_registration(ctx: &Context<'_>, player: &Document) -> Result<(), Error> {
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let players_collection = database.collection::<Document>("Players");
    players_collection
        .delete_one(doc! {"_id": player.get("_id")}, None)
        .await?;
    Ok(())
}
