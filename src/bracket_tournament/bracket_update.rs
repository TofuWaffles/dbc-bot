use std::{process::Stdio, io::Read};
use dbc_bot::{CustomError, QuoteStripper, Region};
use tracing::info;
use crate::{Context, Error};
use crate::database::find::{find_self_by_discord_id, find_enemy_by_match_id_and_self_tag};
use crate::database::config::get_config;
use std::process::Command;
use futures::TryStreamExt;
use base64::{Engine as _, engine::{self, general_purpose}};

pub async fn update_bracket(
    ctx: &Context<'_>,
) -> Result<(), Error> {

    let caller = match find_self_by_discord_id(ctx).await.unwrap() {
        Some(caller) => caller,
        None => {
            info!("Player is not in a tournament, but the function did not return early.");
            return Err(Box::new(CustomError(format!(
                "Player is not in a tournament, but the function did not return early."
            ))));
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

    let config = get_config(ctx, &region).await;

    let mut rounds: Vec<(String, String, String)> = Vec::new();

    for round_number in 1..=config.get("total").unwrap().as_i32().unwrap() {

        let mut database: mongodb::Cursor<mongodb::bson::Document> = ctx.data().database.regional_databases.get(&region).unwrap().collection(format!("Round {}", round_number).as_str()).find(None, None).await?;

        while let Some(current_document) = database.try_next().await? {
            let match_id = current_document
                .get("match_id")
                .and_then(|n| n.as_i32())
                .unwrap();
            let tag = current_document
                .get("tag")
                .and_then(|n| n.as_str())
                .unwrap();
            rounds.push((format!("Round {}", round_number),
                current_document.get("tag").unwrap().to_string(),
                (find_enemy_by_match_id_and_self_tag(ctx, &region, &round_number, &match_id, tag).await).unwrap().get("tag").unwrap().to_string(),
            ));
        }
    }
    
    let output = Command::new("python")
        .arg("bracket_tournament/bracket_generation.py")
        .arg(rounds.iter().map(|(round, player1_tag, player2_tag)| format!("{}|{}|{}", round, player1_tag, player2_tag)).collect::<Vec<String>>().join(","))
        .stdout(Stdio::piped())
        .current_dir("src")
        .spawn()?;

    let mut stdout = output.stdout.ok_or_else(|| Error::from("Failed to capture Python script output"))?;
    let mut buffer = String::new();
    stdout.read_to_string(&mut buffer)?;
    
    let image_bytes = general_purpose::STANDARD.decode(&buffer.trim_end()).unwrap();
    let attachment = poise::serenity_prelude::AttachmentType::Bytes {
        data: image_bytes.into(),
        filename: "Tournament_Bracket.png".to_string(),
    };

    let bracket_channel_data = match (
        config.get("channel_id").and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok())),
        config.get("message_id").and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok())),
    ) {
        (Some(channel_id), Some(message_id)) => (channel_id, message_id),
        _ => {
            info!("Failed to retrieve bracket results channel data.");
            return Err(Box::new(CustomError(format!(
                "Failed to retrieve bracket results channel data."
            ))));
        }
    };

    match poise::serenity_prelude::ChannelId(bracket_channel_data.0.unwrap()).edit_message(&ctx, bracket_channel_data.1.unwrap(), |m| {
        m.attachment(attachment)
    }).await {
        Ok(_) => {},
        Err(err) => {
            return Err(Error::from(err));
        }
    };

    Ok(())
}