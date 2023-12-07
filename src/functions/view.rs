use crate::{
    bracket_tournament::{api, region::Region},
    misc::get_difficulty,
    Context, Error, discord::prompt,
};
use crate::functions::view::prompt::prompt;
use mongodb::bson::Document;
use poise::ReplyHandle;
pub async fn view_info(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    let tag = player.get_str("tag").unwrap();
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    match api::request("player", tag).await {
        Ok(Some(player)) => {
            let club = player["club"]["name"]
                .as_str()
                .map_or("No Club", |name| name);
    
            msg.edit(*ctx, |s| {
                s.components(|c| c)
                .embed(|e| {
                    e.author(|a| a.name(ctx.author().name.clone()))
                    .title(format!("**{} ({})**", player["name"].as_str().unwrap(), player["tag"].as_str().unwrap()))
                    .description("**Here is your information**")
                    .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player["icon"]["id"]))
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
        },
        Ok(None) | Err(_) => {
            prompt(
                ctx,
                msg, 
                "Error: Can't fetch your information",
                "An error occurred while fetching your information. Please try again later.",
                None,
                None
            ).await?;
        }
    };
    
    Ok(())
}
