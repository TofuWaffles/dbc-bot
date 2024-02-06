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
    
    let database = ctx.data().database.regional_databases.get(&Region::find_key(current_region.clone().unwrap().to_string().as_str()).unwrap()).unwrap();
    let collection: mongodb::Collection<mongodb::bson::Document> = database.collection("Config");
    let config = collection.find_one(None, None).await.unwrap().unwrap();
    
    let mut player_data: Vec<(String, String, String, String)> = Vec::new();

    for round_number in 1..=config.get("total").unwrap().as_i32().unwrap() {

        let mut database: mongodb::Cursor<mongodb::bson::Document> = ctx.data().database.regional_databases.get(&current_region.as_ref().unwrap()).unwrap().collection(format!("Round {}", round_number).as_str()).find(None, None).await?;

        while let Some(current_document) = database.try_next().await? {
            let match_id = current_document
                .get("match_id")
                .and_then(|n| n.as_i32())
                .unwrap_or(0);
            let tag = if let Some(tag) = current_document.get("tag").and_then(|n| n.as_str()) {
                tag
            } else {
                continue;
            };
            player_data.push((round_number.to_string(),
                match_id.to_string(),
                current_document.get("tag").unwrap().to_string(),
                (find_enemy_by_match_id_and_self_tag(ctx, region.unwrap(), &round_number, &match_id, tag).await).unwrap().get("tag").unwrap().to_string(),
            ));
        }
    }

    let output = Command::new("python")
        .arg("bracket_tournament/bracket_generation.py")
        .arg(region.unwrap().to_string())
        .arg(config.get("total").unwrap().to_string())
        .arg(match player_data.is_empty() {
            true => " | | | ".to_string(),
            false => player_data.iter().map(|(round, match_id, player1_tag, player2_tag)| format!("{}|{}|{}|{}", round, match_id, player1_tag, player2_tag)).collect::<Vec<String>>().join(",")
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