use crate::bracket_tournament::{api, region::Region};
use crate::database_utils::find_discord_id::find_discord_id;
use crate::misc::{get_difficulty, QuoteStripper};
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::json::Value;
use strum::IntoEnumIterator;

/// A moderator-only command, using required_permissions
#[poise::command(
    slash_command, 
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]

pub async fn get_individual_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] discord_id: String,
) -> Result<(), Error> {
    let data = match find_discord_id(&ctx, Some(discord_id)).await {
        Some(data) => data,
        None => {
            ctx.say("User not found in database").await?;
            return Ok(());
        }
    };
    let region = data.get("region").unwrap().to_string().strip_quote();
    let endpoint = api::get_api_link(
        "player",
        data.get("tag").unwrap().to_string().strip_quote().as_str(),
    );
    let player: Value = api::request(&endpoint).await?;

    ctx.send(|s| {
        s.content("".to_string())
            .reply(true)
            .ephemeral(false)
            .embed(|e| {
                e.author(|a| a.name(ctx.author().name.clone()))
                    .title(format!(
                        "**{} ({})**",
                        &player["name"].to_string().strip_quote(),
                        &player["tag"].to_string().strip_quote()
                    ))
                    .thumbnail(format!(
                        "https://cdn-old.brawlify.com/profile-low/{}.png",
                        player["icon"]["id"]
                    ))
                    .fields(vec![
                        ("**Region**", region.to_string(), true),
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
                        (
                            "Club",
                            player["club"]["name"].to_string().strip_quote(),
                            true,
                        ),
                    ])
                    .timestamp(ctx.created_at())
            })
    })
    .await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn get_all_players_data(ctx: Context<'_>) -> Result<(), Error> {
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let mut player_data: mongodb::Cursor<Document> = database
            .collection("Player")
            .find(None, None)
            .await
            .unwrap();

        let player_data_pages = dashmap::DashMap::<String, Document>::new();

        while let Some(player_data_page) = player_data.try_next().await? {
            let name = player_data_page
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("Username not found.");
            player_data_pages.insert(name.to_string(), player_data_page);
        }

        let page_content = player_data_pages
            .iter()
            .map(|entry| {
                let name = entry.key().clone();
                let data = entry.value().clone();
                let tag = data
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .unwrap_or("Tag not found.");
                let region = data
                    .get("region")
                    .and_then(|r| r.as_str())
                    .unwrap_or("Region not found.");
                let id = data
                    .get("id")
                    .and_then(|i| i.as_str())
                    .unwrap_or("ID not found.");
                format!(
                    "Name: {}\nTag: {}\nRegion: {}\nID: {}\n",
                    name, tag, region, id
                )
            })
            .collect::<Vec<_>>();

        poise::builtins::paginate(
            ctx,
            page_content
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .as_slice(),
        )
        .await?;
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("Reading players information in {}...", region))
            })
            .await?;
    }

    Ok(())
}
