use crate::bracket_tournament::api;
use crate::commands::sample_json;
use crate::misc::{QuoteStripper, CustomError};
use crate::{Context, Error};
use poise::serenity_prelude::json::Value;

const MODE: &str = "wipeout";


///Once the match ends, please run this command to update the result.
#[poise::command(slash_command, guild_only)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error>{
    // Write code to check if the caller has registered the event


    let tag = "#RR82U9J0".to_string();
    let endpoint = api::get_api_link("battle_log", &tag.to_uppercase());
    let raw_logs = api::request(&endpoint).await.unwrap();
    let logs: &Vec<serde_json::Value> = raw_logs["items"].as_array().unwrap();
    let team = sample_json::match_json();
    let enemy: Value;
    let caller = match &tag {
        t if *t == team[0]["tag"].to_string().strip_quote() => {
            enemy = team[1].clone();
            team[0].clone()
        },
        t if *t == team[1]["tag"].to_string().strip_quote() => {
            enemy = team[0].clone();
            team[1].clone()
        },
        _ => panic!("Tag not found in team"),
    };



    println!("Team: {}", team);
    let player1 = team[0]["tag"].to_string().strip_quote();
    println!("Player 1: {}", player1);
    let player2 = team[1]["tag"].to_string().strip_quote();
    println!("Player 2: {}", player2);
    let mut results: Vec<String> = vec![];
    for log in logs.clone(){
        let mode = log["event"]["mode"].to_string().strip_quote();
        let player1_log = log["battle"]["teams"][0][0]["tag"].to_string().strip_quote();
        let player2_log = log["battle"]["teams"][1][0]["tag"].to_string().strip_quote();
        if mode == *MODE
            && (player1 == player1_log || player1 == player2_log)
            && (player2 == player1_log || player2 == player2_log)
        {
            println!("Found the log");
            results.push(log["battle"]["result"].to_string().strip_quote());

        }
    }
    //If there are more than 1 result (best of 2), then we need to check the time
    if results.len() > 1{
        println!("Initialize: is_victory = true");
        let mut is_victory = true;
        let mut count_victory = 0;
        let mut count_defeat = 0;
        
        for result in results.iter().rev() {
            match (*result).strip_quote().as_str(){
                "defeat" => {
                    count_defeat += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        is_victory = false;
                        println!("is_victory = false");
                        break;
                    }
                },
                "victory" => {
                    count_victory += 1;
                    if count_defeat == 2 || count_victory < 2 {
                        println!("is_victory = false");
                        is_victory = false;
                        break;
                    }
                },
                _ => {} // Handle other cases if needed
            }
        }
        println!("Checking is_victory: {}", is_victory);
        if is_victory{ 
            ctx.send(|s|{
                s.reply(true)
                .ephemeral(false)
                .embed(|e|{
                        e.title("The result is in!")
                            .description(format!("Congratulations! {}({}) is ascended to next round!", caller["name"].to_string().strip_quote(), caller["tag"].to_string().strip_quote()))
                    })
            }).await?;
            //Update the database to forward the winnter to next round (use tag)

            //
        }
        else{ 
            ctx.send(|s|{
                s.reply(true)
                .ephemeral(false)
                .embed(|e|{
                    e.title("The result is in!")
                        .description(format!("Congratulations! {}({}) is ascended to next round!", enemy["name"].to_string().strip_quote(), enemy["tag"].to_string().strip_quote()))
                })
            }).await?;
        }

        return Ok(());

    }

    // Certainly L case
    ctx.send(|s| {
        s.content("".to_string())
            .reply(true)
            .ephemeral(false)
            .embed(|e| {
                e.title("No battle logs found (yet?)")
                    .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
            })
    })
    .await?;
    Err(Box::new(CustomError("Unsuccessful response".to_string())))
}
