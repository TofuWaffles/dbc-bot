use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::{api, region::Region};
use crate::checks::user_is_manager;
use crate::database_utils::find_discord_id::find_discord_id;
use crate::database_utils::find_round::get_round;
use crate::misc::{get_difficulty, QuoteStripper, get_icon};
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::options::FindOneOptions;
use poise::serenity_prelude::ReactionType;
use poise::{serenity_prelude as serenity, ReplyHandle};
use poise::serenity_prelude::json::Value;
use tracing::{info, instrument};

/// Checks a player registration status by Discord user ID. Available to mods and sheriffs only.
#[instrument]
#[poise::command(
    context_menu_command = "Player information",
    guild_only,
)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] user: serenity::User,
) -> Result<(), Error> {
    info!("Getting participant data");
    ctx.defer_ephemeral().await?;
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
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
                    &player["name"].as_str().unwrap(),
                    &player["tag"].as_str().unwrap()
                ))
                .thumbnail(format!(
                    "https://cdn-old.brawlify.com/profile-low/{}.png",
                    player["icon"]["id"]
                ))
                .fields(vec![
                    ("**Region**", region.to_string().as_str(), true),
                    ("Trophies", player["trophies"].to_string().as_str(), true),
                    ("Highest Trophies", player["highestTrophies"].to_string().as_str(), true),
                    ("3v3 Victories", player["3vs3Victories"].to_string().as_str(), true),
                    ("Solo Victories", player["soloVictories"].to_string().as_str(), true),
                    ("Duo Victories", player["duoVictories"].to_string().as_str(), true),
                    ("Best Robo Rumble Time", &get_difficulty(&player["bestRoboRumbleTime"]), true),
                    ("Club", player["club"]["name"].as_str().unwrap(), true),
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
    rename="all_participants"
)]
pub async fn get_all_players_data(ctx: Context<'_>, 
    #[description = "Get all participants information in a region in a current round"] region: Region,
    #[description = "(Optional) Only retrieve participants who hasn't finished their match "] no_match: Option<bool> ) -> Result<(), Error>{
    ctx.defer_ephemeral().await?;
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let msg = ctx.send(|s| s.content("Getting player info...").reply(true)).await?;
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    let round = get_round(&config);
    let round_number = round.parse::<i32>().unwrap_or(0);
    let collection = database.collection::<Document>(&round);
    let filter = match no_match{
        Some(true) => doc!{"battle": false},
        _ => doc!{}
    };
    let total = match collection.count_documents(filter.clone(), None).await? as i32{
        0 => {
            msg.edit(ctx, |s| s.content("No players found in database")).await?;
            return Ok(())
        },
        total => total 
    };
    let mut index = 1;
    let first_player = collection.find_one(filter.clone(), None).await?.unwrap();    
    msg.edit(ctx, |s|
        s.components(|c|
            c.create_action_row(|a|
                a.create_button(|b|
                    b.custom_id("previous")
                        .label("Prev")
                        .emoji(ReactionType::Unicode("⬅️".to_string())) // ffs why it has to be so complicated
                        .style(serenity::ButtonStyle::Danger)
                )
                .create_button(|b|
                    b.custom_id("next")
                        .label("Next")
                        .emoji(ReactionType::Unicode("➡️".to_string())) // if only I can use Arius's serenity_utils for this
                        .style(serenity::ButtonStyle::Success)
                )
                
            )
        )
    ).await?;
    get_player_data(&ctx, &msg, &region, first_player, &round_number, &index, &total).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(300));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await{
        match mci.data.custom_id.as_str(){
            "next" => {
                if index == total{
                    index = 1;
                } else {
                    index+=1
                }
            },
            "previous" => {
                if index == 1{
                    index = total;
                } else {
                    index-=1
                };
            },
            _ => unreachable!("This should never happen!")
        }
        let option = FindOneOptions::builder().skip(Some((index-1) as u64)).build();
        let player = collection.find_one(filter.clone(), option).await?.unwrap();
        get_player_data(&ctx, &msg, &region, player, &round_number, &index, &total).await?;
        mci.create_interaction_response(ctx, |ir| {
            ir.kind(serenity::InteractionResponseType::DeferredUpdateMessage)
        })
        .await?;
    }
    Ok(())
}

async fn get_player_data(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region, player: Document, round: &i32, index: &i32, total: &i32) -> Result<(), Error>{
    let name = player.get("name").unwrap().as_str().unwrap();
    let tag = player.get("tag").unwrap().as_str().unwrap();
    let discord_id = player.get("discord_id").unwrap().as_str().unwrap();
    let match_id = match player.get("match_id").unwrap().as_str(){
        Some(match_id) => match_id,
        None => "Not yet assigned."
    };
    let battle = match player.get("battle").unwrap().as_bool().unwrap(){
        true => "Already played",
        false => "Not yet played"
    };
    let icon = player.get("icon").unwrap().to_string();
    let icon_url = get_icon("player")(icon);
    msg.edit(*ctx, |s| {
        s.embed(|e|{
            e.title(format!("Round {} - DBC Tournament Region {}", round, region))
                .description(format!("Player **{} ({})**'s info", name, tag))
                .thumbnail(icon_url)
                .fields(vec![
                    ("**Discord ID**", format!("<@{}>",discord_id), true),
                    ("**Match**", match_id.to_string(), true),
                    ("**Battle**", battle.to_string(), true)
                ])
                .footer(|f|{
                    f.text(format!("{} out of {} players", index, total))
                })
        })
    }).await?;
    Ok(())
}
