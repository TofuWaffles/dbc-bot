use crate::Context;
use dbc_bot::Region;
use mongodb::bson::{doc, Bson::Null, Document};
use mongodb::Collection;
use poise::serenity_prelude::json::Value;

pub fn make_config() -> Document {
    let config = doc! {
      "registration": false,
      "tournament": false,
      "round": 0,
      "mode": Null,
      "map": Null,
      "total": 0,
      "role": Null,
      "channel": Null,
      "bracket_channel": Null
    };
    config
}

pub fn make_player_doc(player: &Value, discord_id: &str, region: &Region) -> Document {
    let name_color = match player["nameColor"] {
        Value::Null => "0xFFFFFFFF",
        _ => player["nameColor"].as_str().unwrap(),
    };
    let player = doc! {
        "name": player["name"].as_str(),
        "name_color": name_color,
        "tag": player["tag"].as_str(),
        "icon": player["icon"]["id"].as_i64(),
        "discord_id": discord_id,
        "region": format!("{:?}", region),
        "match_id": Null,
        "battle": false
    };
    player
}

pub fn set_config(key: &str, value: Option<&str>) -> Document {
    let config = doc! {
        "$set": {
            key: value
        }
    };
    config
}

pub async fn get_config(ctx: &Context<'_>, region: &Region) -> Document {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    collection.find_one(None, None).await.unwrap().unwrap()
}

pub fn toggle_reg_config(status: bool) -> Document {
    let config = doc! {
      "$set": {
        "registration": status
      }
    };
    config
}

pub fn open_tour_close_reg() -> Document {
    let config = doc! {
      "$set": {
        "tournament": true,
        "registration": false,
      }
    };
    config
}


pub fn reset_config() -> Document {
    let config = doc! {
        "$set": {
            "registration": false,
            "tournament": false,
            "round": 0,
            "mode": Null,
            "map": Null,
            "total": 0,
            "role": Null,
            "channel": Null,
            "bracket_channel": Null
        }
    };
    config
}
