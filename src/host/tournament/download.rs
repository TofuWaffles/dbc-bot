use crate::discord::prompt::prompt;
use crate::{database::find::find_all_false_battles, Context, Error};
use dbc_bot::Region;
use futures::{StreamExt, TryStreamExt};
use poise::serenity_prelude::AttachmentType;
use poise::ReplyHandle;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

#[allow(dead_code)]
pub async fn get_downloadable_ids(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Fetching all the id from players who have not played any battle yet",
        "<a:loading:1187839622680690689> Preparing the list...",
        None,
        None,
    )
    .await?;

    let mut download = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("ids.txt")
        .await
    {
        Ok(file) => {
            info!("A new file is created");
            file
        }
        Err(e) => {
            error!("{e}");
            prompt(
                ctx,
                msg,
                "ERROR WRITING FILE",
                format!("{:#?}", e),
                None,
                Some(0xFF0000),
            )
            .await?;
            return Err("Failed to create file".into());
        }
    };
    let mut documents = find_all_false_battles(ctx, region).await;
    while let Some(player) = documents.next().await {
        match player {
            Ok(p) => {
                info!("Writing id: {:?}", p.get_str("discord_id"));
                match p.get_str("discord_id") {
                    Ok(id) => {
                        info!("Writing id: {}", id);
                        match download.write_all(format!("{}\n", id).as_bytes()).await {
                            Ok(_) => continue,
                            Err(e) => {
                                error!("{e}");
                                return prompt(
                                    ctx,
                                    msg,
                                    "ERROR WRITING FILE",
                                    format!("{:#?}", e),
                                    None,
                                    Some(0xFF0000),
                                )
                                .await;
                            }
                        }
                    }
                    Err(_) => {
                        continue; // Mannequin case
                    }
                }
            }
            Err(err) => {
                error!("Error reading document: {}", err);
            }
        }
    }

    let attachment = AttachmentType::File {
        file: &download,
        filename: "ids.txt".to_string(),
    };
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Fetching all the id from players who have not played any battle yet")
                .description("<:tick:1187839626338111600> Done! The list is ready to download")
        })
        .components(|c| c)
        .attachment(attachment)
    })
    .await?;
    Ok(())
}

pub async fn compact(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Fetching all the id from players who have not played any battle yet",
        "<a:loading:1187839622680690689> Preparing the list...",
        None,
        None,
    )
    .await?;
    let mut ids: Vec<String> = vec![];
    let mut documents = find_all_false_battles(ctx, region).await;
    while let Some(player) = documents.next().await {
        match player {
            Ok(p) => {
                if let Ok(id) = p.get_str("discord_id") {
                    ids.push(id.to_string());
                }
            }
            Err(e) => {
                error!("Error reading document: {}", e);
            }
        }
    }
    let pages = chunk::<String>(&ids, 20);
    let mut index = 0;
    
    let content = pages[index].into_iter().map(|id| format!("{id}\n")).collect::<String>();
    msg.edit(*ctx, |b| {
        b.embed(|b| {
            b.description(format!("{}",content))
                .footer(|f| f.text(format!("Page {}/{}", index + 1, pages.len())))
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_button(|b| b.custom_id("prev").emoji('◀'))
                    .create_button(|b| b.custom_id("next").emoji('▶'))
            })
        })
    }).await?;
    
    while let Some(press) = poise::serenity_prelude::CollectComponentInteraction::new(ctx)
    .timeout(std::time::Duration::from_secs(3600 * 24))
    .await{
        match press.data.custom_id.as_str() {
            "prev" => {
                index = if index == 0 { pages.len() - 1 } else { index - 1 };
            }
            "next" => {
                index = if index == pages.len() - 1 { 0 } else { index + 1 };
            }
            _ => {
                continue;
            }
    }
    let content = pages[index].into_iter().map(|id| format!("{id}\n")).collect::<String>();
    msg.edit(*ctx, |b| {
        b.embed(|b| {
            b.description(format!("{}",content))
                .footer(|f| f.text(format!("Page {}/{}", index + 1, pages.len())))
        })
        .components(|b| {
            b.create_action_row(|b| {
                b.create_button(|b| b.custom_id("prev").emoji('◀'))
                    .create_button(|b| b.custom_id("next").emoji('▶'))
            })
        })
    }).await?;
}
    Ok(())
}

fn chunk<T>(slice: &[T], chunk_size: usize) -> Vec<&[T]> {
    slice
        .chunks(chunk_size)
        .map(|chunk| chunk)
        .collect()
}