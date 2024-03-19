use super::config::get_config;
use super::mannequin::add_mannequin;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};

use super::find::{find_enemy_by_match_id_and_self_tag, find_round_from_config};

pub async fn remove_player(
    ctx: &Context<'_>,
    player: &Document,
    region: &Region,
) -> Result<String, Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let config = get_config(ctx, region).await;
    let current_round = find_round_from_config(&config);
    let round_collection = database.collection::<Document>(&current_round);
    let players_collection = database.collection::<Document>("Players");
    match round_collection.name() {
        "Players" => {
            players_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await?;
            Ok(format!(
                "Successfully removed player {}",
                player.get_str("discord_id").unwrap()
            ))
        }
        _ => {
            let match_id = player.get_i32("match_id").unwrap();
            let mannequin = add_mannequin(region, Some(match_id));
            round_collection.insert_one(mannequin, None).await?;
            round_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await?;
            let (_, r_u32) = current_round.split_at(6);
            let next_round_collection = database.collection::<Document>(&format!(
                "Round {}",
                r_u32.trim().parse::<u32>().unwrap_or(0) + 1_u32
            ));
            if next_round_collection
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await
                .is_ok()
            {};
            let enemy = match find_enemy_by_match_id_and_self_tag(
                ctx,
                region,
                &current_round,
                &match_id,
                player.get_str("tag").unwrap(),
            )
            .await
            {
                Some(e) => e,
                None => doc! {
                    "name": "No enemy found",
                    "tag": "No enemy found",
                    "discord_id": 0
                },
            };
            Ok(format!(
                r#"Successfully removed player {player}
Please notify the opponent to run /menu and select "Submit"!
Detail of the opponent:
In-game name: {enemy_name}
Player tag: {enemy_tag}
Discord id: `{enemy_discord_id}`
Mention: <@{enemy_discord_id}>
Match id: {match_id}
"#,
                player = player.get_str("discord_id").unwrap_or(""),
                enemy_name = enemy.get_str("name").unwrap_or(""),
                enemy_tag = enemy.get_str("tag").unwrap_or(""),
                enemy_discord_id = enemy.get_str("discord_id").unwrap_or(""),
                match_id = match_id
            ))
        }
    }
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
