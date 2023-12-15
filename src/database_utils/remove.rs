use crate::Region;
use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::mannequin::add_mannequin;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

use super::find::find_round;

pub async fn remove_player(ctx: &Context<'_>, player: &Document) -> Result<String, Error> {
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let players_collection = database.collection::<Document>("Players");
    players_collection
        .delete_one(doc! {"_id": player.get("_id")}, None)
        .await?;
    let config = get_config(ctx, &region).await;
    let round_collection = ctx
        .data()
        .database
        .regional_databases
        .get(&region)
        .unwrap()
        .collection::<Document>(find_round(&config).as_str());

    match player.get("discord_id").unwrap().as_str() {
        Some(player_id) => {
            let match_id = player
                .get("match_id")
                .unwrap()
                .to_string()
                .parse::<i32>()
                .unwrap();
            let mannequin = add_mannequin(&region, Some(match_id), None);
            round_collection
                .delete_one(doc! {"discord_id": player_id}, None)
                .await?;
            round_collection.insert_one(mannequin, None).await?;
        }
        None => {},
    };
    Ok(find_round(&config))
}
