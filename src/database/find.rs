use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::{
    bson::{doc, Bson, Document},
    Collection, Cursor,
};
use strum::IntoEnumIterator;
use tracing::error;

use super::config::get_config;

pub async fn find_self_by_discord_id(
    ctx: &Context<'_>,
    round: String,
) -> Result<Option<Document>, Error> {
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection(&round);
        let filter = doc! {"discord_id": ctx.author().id.to_string()};
        match collection.find_one(filter, None).await {
            Ok(result) => match result {
                Some(p) => {
                    return Ok(Some(p));
                }
                None => continue,
            },
            Err(_) => {
                return Ok(None);
            }
        }
    }
    Ok(None)
}

pub async fn find_player_by_discord_id(
    ctx: &Context<'_>,
    region: &Region,
    user_id: u64,
    round: &str,
) -> Result<Option<Document>, Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection(round);
    let filter = doc! {"discord_id": user_id.to_string()};
    match collection.find_one(filter, None).await {
        Ok(result) => match result {
            Some(p) => Ok(Some(p)),
            None => Ok(None),
        },
        Err(e) => {
            error!("{e}");
            Ok(None)
        }
    }
}

pub fn find_round_from_config(config: &Document) -> String {
    let round = match config.get("round") {
        Some(round) => {
            if let Bson::Int32(0) = round {
                "Players".to_string()
            } else {
                format!("Round {}", round.as_i32().unwrap())
            }
        }
        _ => unreachable!("Round not found in config!"),
    };

    round
}
/// Asynchronously searches for enemy in the regional databases.
///
/// # Arguments
///
/// - `ctx` - The context of the application.
/// - `region` - The region of the player.
/// - `round` - The round of the match.
/// - `match_id` - The match id of the player.
/// - `other_tag` - The tag of the player.
///
/// # Returns
///
/// An `Option<Document>` representing enemy if found, or `None` if not found or an error occurred.
pub async fn find_enemy_by_match_id_and_self_tag(
    ctx: &Context<'_>,
    region: &Region,
    round: &str,
    match_id: &i32,
    player_tag: &str,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection(round);
    let filter = doc! {
        "match_id": match_id,
        "tag": {"$ne": player_tag}
    };
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => Some(enemy),
        Ok(None) => None,
        Err(_err) => None,
    }
}
/// Asynchronously searches for a discord_id in the regional databases.
pub async fn find_enemy_of_mannequin(
    ctx: &Context<'_>,
    region: &Region,
    round: &str,
    match_id: &i32,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection(round);
    let filter = doc! {
        "match_id": match_id,
        "tag": {"$ne": Bson::Null},
    };
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => Some(enemy),
        Ok(None) => None,
        Err(_err) => None,
    }
}
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
pub async fn find_tag(ctx: &Context<'_>, tag: &str) -> Option<Document> {
    let mut result: Option<Document> = None;
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let player_data: Collection<Document> = database.collection("Players");

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
            Err(_err) => {
                result = None;
                break;
            }
        }
    }
    result
}

pub fn is_mannequin(enemy: &Document) -> bool {
    enemy.get("tag").is_none()
}

pub fn is_disqualified(enemy: &Document) -> bool {
    enemy.get("reason").is_some()
}

pub async fn find_all_false_battles(ctx: &Context<'_>, region: &Region) -> Cursor<Document> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let round = find_round_from_config(&get_config(ctx, region).await);
    let collection: Collection<Document> = database.collection(round.as_str());
    collection.find(doc! {"battle": false}, None).await.unwrap()
}
