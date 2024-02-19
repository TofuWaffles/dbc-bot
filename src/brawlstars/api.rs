use crate::Error;
use poise::serenity_prelude::json::Value;
use reqwest;
use reqwest::StatusCode;
use tracing::info;

pub enum APIResult {
    Successful(Value),
    NotFound(u16),
    APIError(u16),
}
fn get_player(player_tag: &str) -> String {
    format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", player_tag)
}

fn get_battle_log(player_tag: &str) -> String {
    format!(
        "https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog",
        player_tag
    )
}

pub async fn request(option: &str, tag: &str) -> Result<APIResult, Error> {
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };
    let endpoint = match option {
        "player" => get_player(proper_tag),
        "battle_log" => get_battle_log(proper_tag),
        _ => unreachable!("Invalid option"),
    };

    let token = std::env::var("BRAWL_STARS_TOKEN").expect("Brawl Stars API token not found.");

    let response = reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let data: Value = response.json().await?;
            Ok(APIResult::Successful(data))
        }
        StatusCode::NOT_FOUND => Ok(APIResult::NotFound(response.status().as_u16())),
        _ => {
            info!("API error {}", response.status());
            Ok(APIResult::APIError(response.status().as_u16()))
        }
    }
}
