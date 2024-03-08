use serde::{Deserialize, Serialize};
use crate::utils::api::api_handlers;


#[derive(Serialize, Deserialize)]
pub struct Icon{
  pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct Club{
  pub tag: String,
  pub name: String
}

#[derive(Serialize, Deserialize)]
pub struct Player{
  pub tag: String,
  pub name: String,
  pub icon: Icon,
  pub trophies: i32,
  #[serde(rename = "highestTrophies")]
  pub highest_trophies: i32,
  #[serde(rename = "3vs3Victories")]
  pub victories_3v3: i32,
  #[serde(rename = "soloVictories")]
  pub solo_victories: i32,
  #[serde(rename = "duoVictories")]
  pub duo_victories: i32,
  #[serde(rename = "bestRoboRumbleTime")]
  pub best_robo_rumble_time: i32,
  pub club: Club,
}

impl Player {
  pub async fn new(tag: &str) -> Player {
    let endpoint = api_handlers::get_api_link("player", tag);
    let player = api_handlers::request::<Player>(&endpoint).await.unwrap();
    return player;
  }
}

