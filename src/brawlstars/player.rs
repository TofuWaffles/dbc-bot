use crate::{Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;
use super::getters::get_difficulty;
pub async fn stat(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: &serde_json::Value,
    region: &Region,
) -> Result<(), Error> {
    let club = player["club"]["name"]
        .as_str()
        .map_or("No Club", |name| name);
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.author(|a| a.name(ctx.author().name.clone()))
                .title(format!(
                    "**{} ({})**",
                    player["name"].as_str().unwrap(),
                    player["tag"].as_str().unwrap()
                ))
                .description("**Here is your information**")
                .thumbnail(format!("https://cdn-old.brawlify.com/profile/{}.png",player["icon"]["id"]))
                .fields(vec![
                    ("**Region**", format!("{}", region).as_str(), true),
                    ("Trophies", player["trophies"].to_string().as_str(), true),
                    ("Highest Trophies", player["highestTrophies"].to_string().as_str(), true),
                    ("3v3 Victories", player["3vs3Victories"].to_string().as_str(), true),
                    ("Solo Victories", player["soloVictories"].to_string().as_str(), true),
                    ("Duo Victories", player["duoVictories"].to_string().as_str(), true),
                    ("Best Robo Rumble Time", &get_difficulty(&player["bestRoboRumbleTime"]), true),
                    ("Club", club, true),
                ])
                
                .timestamp(ctx.created_at())
        })
    })
    .await?;
    Ok(())
}
