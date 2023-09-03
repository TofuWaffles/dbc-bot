use crate::bracket_tournament::assign_match_id::update_match_id;
use crate::bracket_tournament::{api, region};
use crate::database_utils::find_discord_id::find_discord_id;
use crate::database_utils::find_enemy::find_enemy;
use crate::misc::{CustomError, QuoteStripper};
use crate::{Context, Error};
use mongodb::Collection;
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::json::Value;

const MODE: &str = "wipeout";

///Once the match ends, please run this command to update the result.
#[poise::command(slash_command, guild_only)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {    
    //Getting basic data
    let caller = match find_discord_id(&ctx, None).await{
      Some(caller) => caller,
      None => {
        ctx.send(|s| {
          s.reply(true)
              .ephemeral(false)
              .embed(|e| {
                  e.title("You are not in the tournament!")
                      .description("Sorry, you are not in the tournament to use this command!")
              })
      }).await?;
        return Ok(());
      }
    }; 
    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let region = region::Region::find_key(caller.get("region").unwrap().to_string().strip_quote().as_str()).unwrap();
    //Database set up
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection: Collection<Document> = database.collection("Player");


    let enemy = match find_enemy(&ctx, &region, &match_id, &caller_tag).await{
      Some(enemy) => enemy,
      None => {
        let round2:Collection<Document> = database.collection("Round 2");
        round2.insert_one(update_match_id(caller), None).await.unwrap();
        return Ok(());
      }
    };
    let enemy_tag = enemy.get("tag").unwrap().to_string().strip_quote();

    //Getting battle log data
    let endpoint = api::get_api_link("battle_log", &caller_tag);
    let raw_logs = api::request(&endpoint).await.unwrap();
    let logs: &Vec<Value> = raw_logs["items"].as_array().unwrap();
    let mut results: Vec<String> = vec![];

    for log in logs.clone() {
        let mode = log["event"]["mode"].to_string().strip_quote();
        let player1 = log["battle"]["teams"][0][0]["tag"]
            .to_string()
            .strip_quote();
        let player2 = log["battle"]["teams"][1][0]["tag"]
            .to_string()
            .strip_quote();
        if mode == *MODE
            && (caller_tag == player1 || caller_tag == player2)
            && (enemy_tag == player1 || enemy_tag == player2)
        {
            println!("Found the log");
            results.push(log["battle"]["result"].to_string().strip_quote());
        }
    }
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1 {
        println!("Initialize: is_victory = true");
        let mut is_victory = true;
        let mut count_victory = 0;
        let mut count_defeat = 0;

        for result in results.iter().rev() {
            match (*result).strip_quote().as_str() {
                "defeat" => {
                    count_defeat += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        is_victory = false;
                        println!("is_victory = false");
                        break;
                    }
                }
                "victory" => {
                    count_victory += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        println!("is_victory = false");
                        is_victory = false;
                        break;
                    }
                }
                _ => {} // Handle other cases if needed
            }
        }
        println!("Checking is_victory: {}", is_victory);
        if is_victory {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("The result is in!").description(format!(
                        "Congratulations! {}({}) is ascended to next round!",
                        caller.get("name").unwrap().to_string().strip_quote(),
                        caller_tag
                    ))
                })
            })
            .await?;

            // WRITE CODE TO INSERT ROUND RESULT INTO DB (INCLUDING THE ROUND NUMBER)



            // WRITE CODE TO PASS THE ROUND NUMBER TO UPDATE_BRACKET

        } else {
            ctx.send(|s| {
                s.reply(true).ephemeral(false).embed(|e| {
                    e.title("The result is in!").description(format!(
                        "Congratulations! {}({}) is ascended to next round!",
                        enemy.get("name").unwrap().to_string().strip_quote(),
                        enemy_tag
                    ))
                })
            })
            .await?;

            // WRITE CODE TO INSERT ROUND RESULT INTO DB (INCLUDING THE ROUND NUMBER)



            // WRITE CODE TO PASS THE ROUND NUMBER TO UPDATE_BRACKET

        }

        return Ok(());
    }

    // Certainly L case
    ctx.send(|s| {
        s.reply(true)
            .ephemeral(false)
            .embed(|e| {
                e.title("No battle logs found (yet?)")
                    .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
            })
    })
    .await?;
    Err(Box::new(CustomError("Unsuccessful response".to_string())))
}
