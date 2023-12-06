use crate::{
    bracket_tournament::{api, region::Region},
    commands::index::home,
    discord::menu::registration_menu,
    misc::get_difficulty,
    Context, Error,
};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::ReplyHandle;
const TIMEOUT: u64 = 120;
pub async fn view_info(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    let tag = player.get_str("tag").unwrap();
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    match api::request("player", tag).await {
        Ok(player) => {
            let club = match player["club"]["name"] {
                serde_json::Value::Null => "No Club",
                _ => player["club"]["name"].as_str().unwrap(),
            };
            msg.edit(*ctx,|s| {
                s.components(|c|c)
                .embed(|e| {
                    e.author(|a| a.name(ctx.author().name.clone()))
                    .title(format!("Step 3: Verify the following account: **{} ({})**", player["name"].as_str().unwrap(), player["tag"].as_str().unwrap()))
                    .description("**Please confirm this is the correct account that you are going to use during our tournament!**")
                    .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player["icon"]["id"]))
                    .fields(vec![
                        ("**Region**", format!("{}",region).as_str(), true),
                        ("Trophies", player["trophies"].to_string().as_str(), true),
                        ("Highest Trophies", player["highestTrophies"].to_string().as_str(), true),
                        ("3v3 Victories", player["3vs3Victories"].to_string().as_str(), true),
                        ("Solo Victories", player["soloVictories"].to_string().as_str(), true),
                        ("Duo Victories", player["duoVictories"].to_string().as_str(), true),
                        ("Best Robo Rumble Time", &get_difficulty(&player["bestRoboRumbleTime"]),true),
                        ("Club", club, true),
                    ])
                    .timestamp(ctx.created_at())
                })
        }
      )
        .await?;
        }
        Err(_) => {}
    };
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    Ok(())
}
