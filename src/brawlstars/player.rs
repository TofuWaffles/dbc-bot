use super::getters::get_difficulty;
use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::bson::Document;
use poise::ReplyHandle;
pub async fn stat(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: &serde_json::Value,
    region: &Region,
    detail: Option<&Document>,
) -> Result<(), Error> {
    let club = player["club"]["name"]
        .as_str()
        .unwrap_or("No Club")
        .to_string();
    msg.edit(*ctx, |s| {
        s.components(|c|c)
        .embed(|e| {
            e.author(|a| a.name(ctx.author().name.clone()))
                .title(format!(
                    "**{} ({})**",
                    player["name"].as_str().unwrap(),
                    player["tag"].as_str().unwrap()
                ))
                .description("**Here is your information**")
                .thumbnail(format!(
                    "https://cdn-old.brawlify.com/profile/{}.png",
                    player["icon"]["id"]
                ))
                .fields(vec![
                    ("**Region**", region.full(), true),
                    ("Trophies", player["trophies"].to_string(), true),
                    (
                        "Highest Trophies",
                        player["highestTrophies"].to_string(),
                        true,
                    ),
                    ("3v3 Victories", player["3vs3Victories"].to_string(), true),
                    ("Solo Victories", player["soloVictories"].to_string(), true),
                    ("Duo Victories", player["duoVictories"].to_string(), true),
                    (
                        "Best Robo Rumble Time",
                        get_difficulty(&player["bestRoboRumbleTime"]),
                        true,
                    ),
                    ("Club", club, true),
                    (
                        "Currently in match",
                        detail.map_or_else(
                            || "Not in match".to_string(),
                            |d| d.get_i32("match_id").unwrap_or(0).to_string(),
                        ),
                        true,
                    ),
                    (
                        "Result submit",
                        detail.map_or_else(
                            || "Not in battle".to_string(),
                            |d| {
                                if let Ok(battle) = d.get_bool("battle") {
                                    if battle {
                                        "Yes".to_string()
                                    } else {
                                        "No".to_string()
                                    }
                                } else {
                                    "Undefined".to_string()
                                }
                            },
                        ),
                        true,
                    ),
                    (
                        "Status",
                        detail.map_or_else(
                            || "Not in match".to_string(),
                            |d| {
                                if let Ok(status) = d.get_bool("defeated") {
                                    if status {
                                        "Eliminated".to_string()
                                    } else {
                                        "Advanced".to_string()
                                    }
                                } else {
                                    "Not in the game".to_string()
                                }
                            },
                        ),
                        true,
                    ),
                    (
                        "Ready",
                        detail.map_or_else(
                            || "Not in match".to_string(),
                            |d| {
                                if let Ok(status) = d.get_bool("ready") {
                                    if status {
                                        "Ready".to_string()
                                    } else {
                                        "Not ready".to_string()
                                    }
                                } else {
                                    "Not yet ready".to_string()
                                }
                            },
                        ),
                        true,
                    ),
                ])
                .timestamp(ctx.created_at())
                .color(0x0000FF)
        })
    })
    .await?;
    Ok(())
}
