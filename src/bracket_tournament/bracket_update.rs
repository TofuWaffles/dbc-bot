use std::{process::Stdio, io::Read};
use crate::{Context, Error, misc::QuoteStripper};
use std::process::Command;
use mongodb::bson::{doc, Document};
use futures::TryStreamExt;
use base64::{Engine as _, engine::{self, general_purpose}};

pub async fn update_bracket(
    channel_id: String,
    ctx: &Context<'_>,
) -> Result<(), Error> {
    let mut raw_bracket_data: mongodb::Cursor<Document> = match ctx
        .data()
        .database
        .collection("BracketPair")
        .find(None, None)
        .await
    {
        Ok(result) => result,
        Err(error) => match error.kind.as_ref() {
            mongodb::error::ErrorKind::Command(code) => {
                return Err(Error::from(code.message.to_owned()));
            }
            _ => {
                return Err(Error::from(error));
            }
        },
    };

    let rounds = dashmap::DashMap::<String, Document>::new();

    while let Some(individual_round) = raw_bracket_data.try_next().await? {
        let round = individual_round
            .get("id")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        
        let new_document = doc! {
            "player1_id": individual_round.get("player1_id"),
            "player2_id": individual_round.get("player2_id"),
            "winner_id": individual_round.get("winner_id"),
        };
        
        rounds.insert(round.to_string(), new_document);
    }
    
    let results: Vec<(String, String, String)> = rounds
        .iter()
        .map(|pair| {
            let player1_name = format!("<@{}>", pair.get("player1_id").unwrap());
            let player2_name = format!("<@{}>", pair.get("player2_id").unwrap());
            let result = pair.key().to_string();
            
            (player1_name, player2_name, result)
        })
        .collect();
    
    let results_arg = results
        .iter()
        .map(|(player1, player2, result)| format!("{}|{}|{}", player1, player2, result))
        .collect::<Vec<String>>()
        .join(",");
    
    let output = Command::new("python")
        .arg("bracket_tournament/bracket_generation.py")
        .arg(results_arg)
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

    let bracket_data = ctx
        .data()
        .database
        .collection("Bracket");
        
    let bracket_channel_data: Document = match bracket_data.find_one(
        doc! {
            "channel_id": &channel_id.strip_quote()
        },
        None,
    )
    .await

    {
        Ok(Some(bracket_channel_data)) => bracket_channel_data,
        Ok(None) => {
            panic!("Bracket results channel not found!");
        }
        Err(err) => {
            return Err(Error::from(err));
        }
    };

    let bracket_message = bracket_channel_data.get("message_id").unwrap().to_string().strip_quote().parse::<u64>().unwrap();

    let bracket_channel = poise::serenity_prelude::ChannelId(channel_id.strip_quote().parse::<u64>().unwrap());

    match bracket_channel.edit_message(&ctx, bracket_message, |m| {
        m.attachment(attachment)
    }).await {
        Ok(_) => {},
        Err(err) => {
            return Err(Error::from(err));
        }
    };

    Ok(())
}