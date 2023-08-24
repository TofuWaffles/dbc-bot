use serde::{Deserialize, Serialize};
use crate::utils::api::api_handlers;


#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub id: i64,
    pub mode: String,
    pub map: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Brawler {
    pub id: i64,
    pub name: String,
    pub power: i64,
    pub trophies: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub tag: String,
    pub name: String,
    pub brawler: Brawler,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Battle {
    pub mode: String,
    // #[serde(rename = "type")]
    // pub battle_type: String, // Rename 'type' to 'battle_type' to avoid conflicts
    pub result: String,
    pub duration: i64,
//     #[serde(rename = "trophyChange")]
//     pub trophy_change: i64,
//     #[serde(rename = "starPlayer")]
//     pub star_player: Player,
//     pub teams: Vec<Vec<Player>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    #[serde(rename = "battleTime")]
    pub battle_time: String,
    pub event: Event,
    // pub battle: Battle,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BattleLog {
    pub items: Vec<Item>,
}

impl BattleLog {
  pub async fn new(tag: &str) -> BattleLog {
      let endpoint = api_handlers::get_api_link("battle_log", tag);
      let battle_log = api_handlers::request::<BattleLog>(&endpoint).await.unwrap();
      battle_log
  }

  pub fn get_latest_log(&self) -> &Item{
   &self.items[0]
  }
}

