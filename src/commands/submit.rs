use crate::bracket_tournament::api::{self, api_handlers};
use crate::commands::sample_json;
use crate::utils::embed_color::get_color;
use crate::utils::misc::{get_mode_icon, QuoteStripper};
use crate::{Context, Error};
const MODE: &str = "wipeout";
const MAX_BRAWLER_LEVEL: i32 = 11;

///Once the match ends, please run this command to update the result.
#[poise::command(slash_command, guild_only)]
pub async fn submit(ctx: Context<'_>) -> Result<(), Error> {
    let tag = "RR82U9J0".to_string();
    let log = api_handlers::request(&api_handlers::get_api_link("battle_log", &tag))
        .await
        .unwrap();
    let team = sample_json::match_json();
    println!("Team: {}", team);
    let player1 = team[0]["name"].to_string().strip_quote();
    println!("Player 1: {}", player1);
    let player2 = team[1]["name"].to_string().strip_quote();
    println!("Player 2: {}", player2);
    let mut index = 0;
    let mut result = "".to_string();
    for log_index in 0..=21 {
        let mode = log["items"][log_index]["event"]["mode"]
            .to_string()
            .strip_quote();
        let player1_log = log["items"][log_index]["battle"]["teams"][0][0]["name"]
            .to_string()
            .strip_quote();
        let player2_log = log["items"][log_index]["battle"]["teams"][1][0]["name"]
            .to_string()
            .strip_quote();
        //Number of logs
        if mode == MODE.to_string()
            && (player1 == player1_log || player1 == player2_log)
            && (player2 == player1_log || player2 == player2_log)
        {
            // println!("Found the log for player {} and {}", player1, player2);
            result = log["items"][log_index]["battle"]["result"].to_string();
            break;
        }
        index += 1;
        // println!(
        //     "Battle log {}\nMode: {}\nPlayer 1: {}\nPlayer2: {}",
        //     log_index,
        //     log["items"][log_index]["event"]["mode"]
        //         .to_string()
        //         .strip_quote(),
        //     log["items"][log_index]["battle"]["teams"][0][0]["name"]
        //         .to_string()
        //         .strip_quote(),
        //     log["items"][log_index]["battle"]["teams"][1][0]["name"]
        //         .to_string()
        //         .strip_quote()
        // );
    }
    if index <= 21 {
        ctx.send(|s| {
            s.content("".to_string()).reply(true).embed(|e| {
                e.author(|a| a.name(ctx.author().name.clone()))
                    .title("Battle log result")
                    .color(get_color(result.clone()))
                    .thumbnail(get_mode_icon(&log["items"][index]["event"]["mode"])) // why it no shows?
                    .field(
                        "Battle Time",
                        log["items"][index]["battleTime"].to_string(),
                        false,
                    )
                    .fields(vec![
                        (
                            "Mode",
                            log["items"][index]["event"]["mode"]
                                .to_string()
                                .strip_quote(),
                            true,
                        ),
                        (
                            "Map",
                            log["items"][index]["event"]["map"]
                                .to_string()
                                .strip_quote(),
                            true,
                        ),
                        (
                            "Duration",
                            log["items"][index]["battle"]["duration"]
                                .to_string()
                                .strip_quote()
                                + "s",
                            true,
                        ),
                        (
                            "Game",
                            log["items"][index]["battle"]["type"]
                                .to_string()
                                .strip_quote(),
                            true,
                        ),
                        (
                            "Trophy Change",
                            log["items"][index]["battle"]["trophyChange"]
                                .to_string()
                                .strip_quote(),
                            true,
                        ),
                        ("", "".to_string(), false),
                    ])
                    .field("===============================".to_string(), "", false)
                    .fields(vec![
                        (
                            log["items"][index]["battle"]["teams"][0][0]["name"]
                                .to_string()
                                .strip_quote(),
                            format!(
                                "Brawler: {}\nPower: {}",
                                &log["items"][index]["battle"]["teams"][0][0]["brawler"]["name"]
                                    .to_string()
                                    .strip_quote(),
                                MAX_BRAWLER_LEVEL
                            ),
                            true,
                        ),
                        ("VS".to_string(), "".to_string(), true),
                        (
                            log["items"][index]["battle"]["teams"][1][0]["name"]
                                .to_string()
                                .strip_quote(),
                            format!(
                                "Brawler: {}\nPower: {}",
                                &log["items"][index]["battle"]["teams"][1][0]["brawler"]["name"]
                                    .to_string()
                                    .strip_quote(),
                                MAX_BRAWLER_LEVEL
                            ),
                            true,
                        ),
                    ])
            })
        })
        .await?;
        return Ok(());
    }
    ctx.send(|s| {
        s.content("".to_string())
            .reply(true)
            .ephemeral(false)
            .embed(|e| {
                e.title("No battle log found (yet?)")
                    .description("As the result is recorded nearly in real-time, please try again later. It may take up to 30 minutes for a new battle to appear in the battlelog")
            })
    })
    .await?;
    return Err(Box::new(api_handlers::CustomError(
        "Unsuccessful response".to_string(),
    )));
}
