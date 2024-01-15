use super::config::get_config;
use super::mannequin::add_mannequin;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

use super::find::find_round_from_config;

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
        .collection::<Document>(find_round_from_config(&config).as_str());

        let player_id = player.get_str("discord_id").unwrap();

        {
            let match_id = player
                .get("match_id")
                .unwrap()
                .to_string()
                .parse::<i32>()
                .unwrap();
        
            let mannequin = add_mannequin(&region, Some(match_id));
        
            round_collection
                .delete_one(doc! {"discord_id": player_id}, None)
                .await?;
        
            round_collection.insert_one(mannequin, None).await?;
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
