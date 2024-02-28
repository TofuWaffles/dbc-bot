use crate::bracket_tournament::bracket_update::update_bracket;
use crate::brawlstars::api::{self, APIResult};
use crate::database::battle::battle_happened;
use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_round_from_config, find_self_by_discord_id,
    is_mannequin,
};
use crate::database::update::update_battle;
use crate::database::update::update_match_id;
use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::{QuoteStripper, Region};
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use poise::serenity_prelude::ChannelId;
use poise::ReplyHandle;
use tracing::info;

/// If you are a participant, run this command once you have finished your match round.
///
/// Automatically grabs the user's match result from the game and updates the bracket.

pub async fn submit_result(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| s.content("Checking your match result..."))
        .await?;
    let round = find_round_from_config(&get_config(ctx, region).await);
    //Check if the user is in the tournament
    let caller = match find_self_by_discord_id(ctx, round).await.unwrap() {
        Some(caller) => caller,
        None => {
            return prompt(
                ctx,
                msg,
                "Sorry, you are not in the tournament!",
                "You have to be in a tournament to use this command!",
                None,
                Some(0xFF0000),
            )
            .await;
        }
    };
    let region = Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(ctx, &region).await;

    //Get player document via their discord_id
    let match_id: i32 = caller.get_i32("match_id").unwrap();
    let caller_tag = caller.get_str("tag").unwrap();
    //Check if the user has already submitted the result or not yet disqualified

    let mode = config.get_str("mode").unwrap();
    let map = config.get_str("map").unwrap_or_default();
    let current_round: Collection<Document> =
        database.collection(find_round_from_config(&config).as_str());
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(ctx, caller_tag, current_round, msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy = find_enemy_by_match_id_and_self_tag(ctx, &region, &round, &match_id, caller_tag)
        .await
        .unwrap();
    if is_mannequin(&enemy) {
        let next_round = database.collection(format!("Round {}", round + 1).as_str());
        next_round.insert_one(update_match_id(caller), None).await?;
        prompt(
            ctx,
            msg,
            "Bye... See you next round",
            "
            Congratulation, you pass this round!",
            None,
            Some(0xFFFF00),
        )
        .await?;
        update_battle(database, round, match_id).await?;
        update_bracket(ctx, None).await?;
        return Ok(());
    }
    println!("{:?}", config);
    let channel = config
        .get("channel")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let bracket_msg_id = config.get_str("bracket_message_id").unwrap();
    let bracket_chn_id = config.get_str("bracket_channel").unwrap();
    let server_id = ctx.guild_id().unwrap().0;
    let channel_to_announce = ChannelId(channel);
    match get_result(mode, map, caller, enemy).await {
        Some(winner) => {
            if round < config.get("total").unwrap().as_i32().unwrap() {
                let next_round: Collection<Document> =
                    database.collection(format!("Round {}", round + 1).as_str());
                next_round
                    .insert_one(update_match_id(winner.clone()), None)
                    .await?;
                update_battle(database, round, match_id).await?;
                update_bracket(ctx, None).await?;
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!")
                            .description(format!(
                                r#"{}({}) has won this round!
The [bracket](https://discord.com/channels/{guild}/{chn}/{msg_id}) is updated"#,
                                winner.get_str("name").unwrap(),
                                winner.get_str("tag").unwrap(),
                                guild = server_id,
                                chn = bracket_chn_id,
                                msg_id = bracket_msg_id
                            ))
                            .color(0xFFFF00)
                    })
                    .components(|c| c)
                })
                .await?;
                channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!").description(format!(
                                r#"{}({}) has won this round!
                                The [bracket](https://discord.com/channels/{guild}/{chn}/{msg_id}) is updated"#,
                                                            winner.get_str("name").unwrap(),
                                                            winner.get_str("tag").unwrap(),
                                                            guild = server_id,
                                                            chn = bracket_chn_id,
                                                            msg_id = bracket_msg_id
                                                        ))
                        .color(0xFFFF00)})
                    })
                    .await?;
            } else {
                database
                    .collection::<Collection<Document>>(format!("Round {}", round).as_str())
                    .update_one(
                        doc! { "_id": winner.get_object_id("_id")? },
                        doc! { "$set": { "winner": true } },
                        None,
                    )
                    .await?;
                update_battle(database, round, match_id).await?;
                update_bracket(ctx, None).await?;
                msg.edit(*ctx, |s| {
                    s.embed(|e| {
                        e.title("Result is here!").description(format!(
                            "CONGRATULATIONS! {}({}) IS THE TOURNAMENT CHAMPION!",
                            winner.get_str("name").unwrap(),
                            winner.get_str("tag").unwrap()
                        ))
                    })
                    .components(|c| c)
                })
                .await?;
                channel_to_announce
                    .send_message(ctx, |m| {
                        m.embed(|e| {
                            e.title("Result is here!").description(format!(
                                "CONGRATULATIONS! {}({}) IS THE TOURNAMENT CHAMPION!",
                                winner.get_str("name").unwrap(),
                                winner.get_str("tag").unwrap()
                            ))
                        })
                    })
                    .await?;
            }
        }
        None => {
            ctx.send(|s| {
                s.embed(|e| {
                        e.title("There are not enough results yet!")
                            .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 seconds for a new battle to appear in the battlelog")              
                    })
                    .components(|c|c)
            }).await?;
        }
    }
    Ok(())
}

async fn get_result(mode: &str, map: &str, caller: Document, enemy: Document) -> Option<Document> {
    let caller_tag = caller.get("tag").unwrap().as_str().unwrap();
    let enemy_tag = enemy.get("tag").unwrap().as_str().unwrap();
    let logs = match api::request("battle_log", caller_tag).await {
        Ok(APIResult::Successful(battle_log)) => {
            Some(battle_log["items"].as_array().unwrap().clone())
        }
        Ok(APIResult::APIError(_)) => None,
        Ok(APIResult::NotFound(_)) | Err(_) => None,
    };
    let mut results: Vec<String> = vec![];

    for log in logs.unwrap() {
        let mode_log = log["event"]["mode"].as_str().unwrap();
        let map_log = log["event"]["map"].as_str().unwrap();
        if !compare_strings(log["battle"]["type"].as_str().unwrap(), "friendly") || !compare_strings(mode_log, mode) {
            continue;
        } 
        if !map.is_empty() && !compare_strings(map_log, map){
            continue;
        }
        
        let player1 = log["battle"]["teams"][0][0]["tag"].as_str().unwrap();
        let player2 = log["battle"]["teams"][1][0]["tag"].as_str().unwrap();
        if (compare_tag(caller_tag, player1) || compare_tag(caller_tag, player2))
            && (compare_tag(enemy_tag, player1) || compare_tag(enemy_tag, player2))
        {
            results.push(log["battle"]["result"].as_str().unwrap().to_string());
        }
        
    }
    info!("{:?}", results);
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1 {
        let mut is_victory: Option<bool> = None;
        let mut count_victory = 0;
        let mut count_defeat = 0;

        for result in results.iter().rev() {
            match result.as_str() {
                "defeat" => count_defeat += 1,
                "victory" => count_victory += 1,
                _ => {} // Handle other cases if needed
            }

            if count_defeat == 2 && count_victory < 2 {
                is_victory = Some(false);
                break;
            } else if count_victory >= 2 {
                is_victory = Some(true);
                break;
            }
        }
        match is_victory {
            Some(true) => Some(caller),
            Some(false) => Some(enemy),
            None => None,
        }
    } else {
        None
    }
}

fn compare_tag(s1: &str, s2: &str) -> bool {
    s1.chars()
        .zip(s2.chars())
        .all(|(c1, c2)| c1 == c2 || (c1 == 'O' && c2 == '0') || (c1 == '0' && c2 == 'O'))
        && s1.len() == s2.len()
}

fn compare_strings(str1: &str, str2: &str) -> bool {
    // Remove punctuation and convert to lowercase
    let str1_normalized = str1.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();

    let str2_normalized = str2.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    str1_normalized == str2_normalized
}