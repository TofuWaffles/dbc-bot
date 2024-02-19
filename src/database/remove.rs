use super::config::get_config;
use super::mannequin::add_mannequin;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

use super::find::find_round_from_config;

pub async fn remove_player(
    ctx: &Context<'_>,
    player: &Document,
    region: &Region,
) -> Result<String, Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let config = get_config(ctx, region).await;
    let round_collection =
        database.collection::<Document>(find_round_from_config(&config).as_str());
    let players_collection = database.collection::<Document>("Players");
    match round_collection.name() {
        "Players" => {
            players_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await?;
        }
        _ => {
            let match_id = player.get_i32("match_id").unwrap();
            let mannequin = add_mannequin(region, Some(match_id));
            round_collection.insert_one(mannequin, None).await?;
            round_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await?;
        }
    }
    Ok(find_round_from_config(&config))
}

pub async fn remove_registration(ctx: &Context<'_>, player: &Document) -> Result<(), Error> {
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let players_collection = database.collection::<Document>("Players");
    players_collection
        .delete_one(doc! {"_id": player.get("_id")}, None)
        .await?;
    Ok(())
}
