use crate::utils::api::api_handlers;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    id: i64,
    mode: String,
    map: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Brawler {
    id: i64,
    name: String,
    power: i64,
    trophies: i64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    tag: String,
    name: String,
    brawler: Brawler,
}

#[derive(Debug, Deserialize, Serialize)]
struct Battle {
    mode: String,
    #[serde(rename = "type")]
    battle_type: String, // Rename 'type' to 'battle_type' to avoid conflicts
    result: String,
    duration: i64,
    #[serde(rename = "trophyChange")]
    trophy_change: i64,
    #[serde(rename = "starPlayer")]
    star_player: Player,
    teams: Vec<Vec<Player>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Item {
    #[serde(rename = "battleTime")]
    battle_time: String,
    event: Event,
    battle: Battle,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BattleLog {
    items: Vec<Item>,
}

impl BattleLog {
    pub async fn new(tag: &str) -> BattleLog {
        let endpoint = api_handlers::get_api_link("battle_log", tag);
        let battle_log = api_handlers::request::<BattleLog>(&endpoint).await.unwrap();
        return battle_log;
    }
    pub fn get_latest_log(&self) -> String {
        return format!("{:?}", self.items[0]);
    }
}
