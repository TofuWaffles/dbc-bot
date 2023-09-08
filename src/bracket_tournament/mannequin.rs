use mongodb::bson::{doc, Bson::Null, Document};

use super::region::Region;

pub fn add_mannequin(
    region: &Region,
    match_id: Option<i32>,
    discord_id: Option<String>,
) -> Document {
    let discord_id: mongodb::bson::Bson = match discord_id {
        Some(id) => mongodb::bson::Bson::String(id),
        None => Null,
    };
    let match_id: mongodb::bson::Bson = match match_id {
        Some(id) => mongodb::bson::Bson::Int32(id),
        None => Null,
    };
    let mannequin = doc! {
      "name": "Mannequin",
      "tag": Null,
      "discord_id:": discord_id,
      "region": format!("{:?}", *region),
      "match_id": match_id,
      "battle": false
    };
    mannequin
}

pub fn update_mannequin(match_id: i32) -> Document{
    let mannequin = doc! {
        "$set": {
            "match_id": match_id
        }
    };
    mannequin
}
