use std::{process::Stdio, io::Read};
use dbc_bot::{CustomError, QuoteStripper, Region};
use mongodb::bson::doc;
use tracing::{error, info};
use crate::{Context, Error};
use crate::database::find::{find_self_by_discord_id, find_enemy_by_match_id_and_self_tag};
use crate::database::config::set_config;
use std::process::Command;
use futures::TryStreamExt;
use base64::{Engine as _, engine::{self, general_purpose}};

pub async fn update_bracket(
    ctx: &Context<'_>,
    region: Option<&Region>,
) -> Result<(), Error> {

    let mut current_region: Option<Region> = None;

    match region {
        Some(region) => {
            current_region = Some(region.clone());
        }
        None => {
            let caller = match find_self_by_discord_id(ctx).await.unwrap() {
                Some(caller) => caller,
                None => {
                    info!("Player is not in a tournament, but the function did not return early.");
                    return Err(Box::new(CustomError(format!(
                        "Player is not in a tournament, but the function did not return early."
                    ))));
                }
            };
            current_region = Region::find_key(
                caller
                    .get("region")
                    .unwrap()
                    .to_string()
                    .strip_quote()
                    .as_str()
                    .as_ref(),
            )
        }
    }
    
    let database = ctx.data().database.regional_databases.get(&current_region.as_ref().unwrap()).unwrap();
    let collection: mongodb::Collection<mongodb::bson::Document> = database.collection("Config");
    let config = collection.find_one(None, None).await.unwrap().unwrap();
    
    let mut player_data: Vec<(i32, i32, String, String, bool, bool)> = Vec::new();
    let mut match_ids = Vec::new();

    for round_number in 1..=config.get("total").unwrap().as_i32().unwrap() {

        let mut database: mongodb::Cursor<mongodb::bson::Document> = ctx.data().database.regional_databases.get(&current_region.as_ref().unwrap()).unwrap().collection(format!("Round {}", round_number).as_str()).find(None, None).await?;

        while let Some(current_document) = database.try_next().await? {
            let match_id = current_document
                .get("match_id")
                .and_then(|n| n.as_i32())
                .unwrap_or(0);
            if match_ids.contains(&match_id) {
                continue;
            }
            let tag = if let Some(tag) = current_document.get("tag").and_then(|n| n.as_str()) {
                tag
            } else {
                continue;
            };
            match_ids.push(match_id);
            player_data.push((round_number,
                match_id,
                current_document.get("name").map_or(" ".to_string(), |name| name.to_string().strip_quote()),
                (find_enemy_by_match_id_and_self_tag(ctx, current_region.as_ref().unwrap(), &round_number, &match_id, tag).await).map_or(" ".to_string(), |document| document.get("name").unwrap().to_string().strip_quote()),
                current_document.get("winner").map_or(false, |is_winner| is_winner.as_bool().unwrap()),
                (find_enemy_by_match_id_and_self_tag(ctx, current_region.as_ref().unwrap(), &round_number, &match_id, tag).await).map_or(false, |document| document.get("winner").map_or(false, |is_winner| is_winner.as_bool().unwrap()))
            ));
        }
        match_ids.clear();
    }

    let output = Command::new("python")
        .arg("bracket_tournament/bracket_generation.py")
        .arg(current_region.as_ref().unwrap().to_string())
        .arg(config.get("total").unwrap().to_string())
        .arg(match player_data.is_empty() {
            true => "1|1| | | | ".to_string(),
            false => player_data.iter().map(|(round, match_id, player1_tag, player2_tag, is_winner1, is_winner2)| format!("{}|{}|{}|{}|{}|{}", round, match_id, player1_tag, player2_tag, is_winner1, is_winner2)).collect::<Vec<String>>().join(",")
        })
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

    match config.get("bracket_channel").and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok()))
    {
        Some(channel_id) => {
            match config.get("bracket_message_id").and_then(|v| v.as_str().map(|s| s.parse::<u64>().ok()))
            {
                Some(bracket_message_id) => {
                    match poise::serenity_prelude::ChannelId(channel_id.unwrap()).edit_message(&ctx, bracket_message_id.unwrap(), |m| {
                        m.attachment(attachment)
                    }).await {
                        Ok(_) => {},
                        Err(err) => {
                            return Err(Error::from(err));
                        }
                    }
                },
                None => {
                    match poise::serenity_prelude::ChannelId(channel_id.unwrap()).send_message(&ctx, |m| {
                        m.add_file(attachment)
                    }).await {
                        Ok(message) => {
                            match collection
                            .update_one(doc! {}, set_config("bracket_message_id", Some(message.id.to_string().as_str())), None)
                            .await {
                                Ok(_) => {}
                                Err(err) => match err.kind.as_ref() {
                                    mongodb::error::ErrorKind::Command(code) => {
                                        error!("Command error: {:?}", code);
                                    }
                                    mongodb::error::ErrorKind::Write(code) => {
                                        error!("Write error: {:?}", code);
                                    }
                                    _ => {
                                        error!("Error: {:?}", err);
                                    }
                                },
                            }
                        },
                        Err(err) => {
                            return Err(Error::from(err));
                        }
                    }
                }
                _ => {
                    info!("Failed to retrieve bracket results channel data.");
                    return Err(Box::new(CustomError(format!(
                        "Failed to retrieve bracket results channel data."
                    ))));
                }
            }
        }
        _ => {
            info!("Failed to retrieve bracket results channel data.");
            return Err(Box::new(CustomError(format!(
                "Failed to retrieve bracket results channel data."
            ))));
        }
    };

    Ok(())
}