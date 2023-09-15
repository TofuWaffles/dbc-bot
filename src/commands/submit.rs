use crate::bracket_tournament::{
    api, config::get_config, match_id::update_match_id, region, update_battle::update_battle,
};
use crate::database_utils::battle_happened::battle_happened;
use crate::database_utils::find_discord_id::find_discord_id;
use crate::database_utils::find_enemy::{find_enemy, is_mannequin};
use crate::database_utils::find_round::get_round;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use mongodb::Collection;
use poise::serenity_prelude::json::Value;
use tracing::{info, instrument};

/// If you are a participant, run this command once you have finished your match round.
///
/// Automatically grabs the user's match result from the game and updates the bracket.
#[instrument]
#[poise::command(slash_command, guild_only, rename = "submit-result")]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {
    info!("Checking user {}'s match result", ctx.author().tag());
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Checking your match result...")
        })
        .await?;
    //Check if the user is in the tournament
    let caller = match find_discord_id(&ctx, None, None).await {
        Some(caller) => caller,
        None => {
            msg.edit(ctx, |s| {
                s.embed(|e| {
                    e.title("Sorry, you are not in the tournament!")
                        .description("You have to be in a tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };

    info!("Getting player document from Discord ID");
    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let region = region::Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    //Check if the user has already submitted the result or not yet disqualified
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    let mode = config.get("mode").unwrap().to_string().strip_quote();
    let map = config.get("map").unwrap().to_string().strip_quote();
    let current_round: Collection<Document> =
        database.collection(get_round(&config).as_str());
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(&ctx, &caller_tag, current_round, &msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy = find_enemy(&ctx, &region, &round, &match_id, &caller_tag)
        .await
        .unwrap();
    if is_mannequin(&enemy) {
        let next_round = database.collection(format!("Round {}", round + 1).as_str());
        next_round.insert_one(update_match_id(caller), None).await?;
        msg.edit(ctx, |s| {
            s.embed(|e| {
                e.title("Bye! See you next... round!").description(
                    "You have been automatically advanced to the next round due to bye!",
                )
            })
        })
        .await?;
        update_battle(database, round, match_id).await?;
        return Ok(());
    }

    match get_result(mode, map, caller, enemy).await {
        Some(winner) => {
            let next_round: Collection<Document> =
                database.collection(format!("Round {}", round + 1).as_str());
            next_round
                .insert_one(update_match_id(winner.clone()), None)
                .await?;
            update_battle(database, round, match_id).await?;
            msg.edit(ctx, |s| {
                s.embed(|e| {
                    e.title("Result is here!").description(format!(
                        "{}({}) has won this round! You are going to next round!",
                        winner.get("name").unwrap().to_string().strip_quote(),
                        winner.get("tag").unwrap().to_string().strip_quote()
                    ))
                })
            })
            .await?;
        }
        None => {
            ctx.send(|s| {
                s.reply(true)
                .ephemeral(true)
                    .embed(|e| {
                        e.title("There are not enough results yet!")
                            .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
                    })
            })
            .await?;
        }
    }
    Ok(())
}

async fn get_result(
    mode: String,
    _map: String,
    caller: Document,
    enemy: Document,
) -> Option<Document> {
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let enemy_tag = enemy.get("tag").unwrap().to_string().strip_quote();
    let endpoint = api::get_api_link("battle_log", &caller_tag);
    let raw_logs = api::request(&endpoint).await.unwrap();
    let logs: &Vec<Value> = raw_logs["items"].as_array().unwrap();
    let mut results: Vec<String> = vec![];

    for log in logs.clone() {
        let mode_log = log["event"]["mode"].to_string().strip_quote();
        let player1 = log["battle"]["teams"][0][0]["tag"]
            .to_string()
            .strip_quote();
        let player2 = log["battle"]["teams"][1][0]["tag"]
            .to_string()
            .strip_quote();
        if mode_log == mode.strip_quote()
            && (caller_tag == player1 || caller_tag == player2)
            && (enemy_tag == player1 || enemy_tag == player2)
        {
            results.push(log["battle"]["result"].to_string().strip_quote());
        }
    }
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1 {
        let mut is_victory: Option<bool> = None;
        let mut count_victory = 0;
        let mut count_defeat = 0;

        for result in results.iter().rev() {
            match result.strip_quote().as_str() {
                "defeat" => count_defeat += 1,
                "victory" => count_victory += 1,
                _ => {} // Handle other cases if needed
            }

            if count_defeat == 2 && count_victory < 2 {
                is_victory = Some(false);
                break;
            } else if count_victory == 2 {
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
