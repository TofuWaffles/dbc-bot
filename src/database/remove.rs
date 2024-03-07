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
    let current_round = find_round_from_config(&config);
    let round_collection =
        database.collection::<Document>(&current_round);
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
            let (_, r_u32) = current_round.split_at(6);
            let next_round_collection = database.collection::<Document>(&format!("Round {}",r_u32.trim().parse::<u32>().unwrap_or(0)+1_u32));
            match next_round_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await{ // in case we disqualify a player who already moved to the next round
                    Ok(_) => {}
                    Err(_) => {}
                };
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
