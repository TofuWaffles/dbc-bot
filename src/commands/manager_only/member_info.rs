use crate::bracket_tournament::{api, region::Region};
use crate::database_utils::find_discord_id::find_discord_id;
use crate::misc::{get_difficulty, QuoteStripper};
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::json::Value;
use tracing::{info, instrument};

/// Checks a player registration status by Discord user ID. Available to mods and sheriffs only.
#[instrument]
#[poise::command(
    context_menu_command = "Player information",
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] user: serenity::User,
) -> Result<(), Error> {
    info!("Getting participant data");
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| s.content("Getting player info...").reply(true))
        .await?;
    let discord_id = user.id.to_string();
    let data = match find_discord_id(&ctx, Some(discord_id), None).await {
        Some(data) => data,
        None => {
            msg.edit(ctx, |s| s.content("User not found in database"))
                .await?;
            return Ok(());
        }
    };
    let region = data.get("region").unwrap().to_string().strip_quote();
    let player: Value = api::request("player", data.get("tag").unwrap().as_str().unwrap()).await?;

    msg.edit(ctx, |s| {
        s.content("".to_string()).embed(|e| {
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
    info!("Successfully retrieved participant data");
    Ok(())
}

/// Checks all players' registration status by Discord user ID. Available to mods and sheriffs only.
#[instrument]
#[poise::command(
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
    rename="all_participants"
)]
pub async fn get_all_players_data(ctx: Context<'_>, region: Region) -> Result<(), Error> {
    info!("Getting all participants' data");
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let mut player_data = match database
        .collection::<Document>("Player")
        .find(None, None)
        .await
    {
        Ok(player_data) => player_data,
        Err(_) => {
            ctx.say(format!(
                "Error occurred while finding player data for {}",
                region
            ))
            .await?;
            return Ok(());
        }
    };

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
                .get("discord_id")
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

    info!("Successfully retrieved all participants' data");

    Ok(())
}
