use crate::brawlstars::getters::get_player_icon;
use crate::database::config::get_config;
use crate::database::find::find_round_from_config;
use crate::Region;
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use poise::ReplyHandle;
use tracing::info;
const TIMEOUT: u64 = 120;
pub async fn detail(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    info!(
        "{} is checking players in detail command at region {}!",
        ctx.author().name,
        region.full()
    );
    msg.edit(*ctx, |s| {
        s.embed(|e| e.description("Getting player info..."))
    })
    .await?;
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let config = get_config(ctx, region).await;
    let round = find_round_from_config(&config);
    let collection = database.collection::<Document>(&round);
    info!("Round: {round}");
    let total = match collection.count_documents(doc! {}, None).await? as i32 {
        0 => {
            msg.edit(*ctx, |s| {
                s.embed(|e| e.description("No player found in database."))
                    .components(|c| c)
            })
            .await?;
            return Ok(());
        }
        total => total,
    };
    let mut index = 1;
    let first_player = collection.find_one(doc! {}, None).await?.unwrap();
    msg.edit(*ctx, |s| {
        s.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("previous")
                        .label("Prev")
                        .emoji(poise::serenity_prelude::ReactionType::Unicode(
                            "⬅️".to_string(),
                        ))
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                })
                .create_button(|b| {
                    b.custom_id("next")
                        .label("Next")
                        .emoji(poise::serenity_prelude::ReactionType::Unicode(
                            "➡️".to_string(),
                        ))
                        .style(poise::serenity_prelude::ButtonStyle::Success)
                })
            })
        })
    })
    .await?;
    get_player_data(ctx, msg, region, first_player, &round, &index, &total).await?;
    let resp = msg.clone().into_message().await?;

    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "next" => match index {
                _ if index == total => index = 1,
                _ => index += 1,
            },
            "previous" => match index {
                1 => index = total,
                _ => index -= 1,
            },
            _ => {
                continue;
            }
        }
        mci.defer(&ctx.http()).await?;
        let option = mongodb::options::FindOneOptions::builder()
            .skip(Some((index - 1) as u64))
            .build();
        let filter = doc! { "discord_id": { "$ne": null } };
        let player = collection.find_one(filter, option).await?.unwrap();
        get_player_data(ctx, msg, region, player, &round, &index, &total).await?;
    }
    Ok(())
}

async fn get_player_data(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    player: mongodb::bson::Document,
    round: &str,
    index: &i32,
    total: &i32,
) -> Result<(), Error> {
    let name = player.get_str("name").unwrap_or("Unknown player");
    let tag = player.get_str("tag").unwrap();
    let discord_id = player.get_str("discord_id").unwrap();
    let match_id = player
        .get_i32("match_id")
        .map_or_else(|_| "Not yet assigned".to_string(), |id| id.to_string());
    let battle = match player.get_bool("battle").unwrap() {
        true => "Already played",
        false => "Not yet played",
    };
    let icon = player.get_i64("icon").unwrap();
    let icon_url = get_player_icon(icon);
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title(format!(
                "Round {} - DBC Tournament Region {}",
                round, region
            ))
            .description(format!("Player **{} ({})**'s info", name, tag))
            .thumbnail(icon_url)
            .fields(vec![
                ("**Discord ID**", format!("<@{}>", discord_id), true),
                ("**Match**", match_id, true),
                ("**Battle**", battle.to_string(), true),
            ])
            .footer(|f| f.text(format!("{index} out of {total} players.")))
        })
    })
    .await?;
    Ok(())
}

pub fn term(status: bool) -> String {
    if status {
        "Open".to_string()
    } else {
        "Closed".to_string()
    }
}
