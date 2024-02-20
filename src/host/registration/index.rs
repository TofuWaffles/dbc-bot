use crate::brawlstars::getters::get_player_icon;
use crate::database::config::get_config;
use crate::database::find::find_round_from_config;
use crate::database::open::{registration_open, tournament};
use crate::database::stat::count_registers;
use crate::database::update::toggle_registration;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::doc;
use poise::ReplyHandle;
const TIMEOUT: u64 = 300;
struct Reg {
    registration: bool,
    tournament: bool,
    count: i32,
    region: Region,
}

pub async fn registration_mod_panel(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let mut reg = getter(ctx, region).await?;
    display_info(ctx, msg, &reg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "registration" => {
                mci.defer(&ctx.http()).await?;
                toggle_registration(ctx, region, !reg.registration).await?;
            }
            "detail" => {
                mci.defer(&ctx.http()).await?;
                detail(ctx, msg, region).await?;
            }
            _ => {
                reg = getter(ctx, region).await?;
                display_info(ctx, msg, &reg).await?;
                continue;
            }
        }
        reg = getter(ctx, region).await?;
        display_info(ctx, msg, &reg).await?;
    }
    Ok(())
}

async fn display_info(ctx: &Context<'_>, msg: &ReplyHandle<'_>, reg: &Reg) -> Result<(), Error> {
    let flag = if reg.tournament {
        "\n. Tournament is currently running. Toggle is disabled!"
    } else {
        ""
    };
    msg.edit(*ctx, |m| {
      m.embed(|e| {
          e.title("**Registration Panel**")
              .description(format!("Registration is currently: {}\nThere are {} registered players for the tournament of {}.{}", term(reg.registration), reg.count, reg.region, flag))
              .image("")
      })
      .components(|c|{
        c.create_action_row(|row| {
          row.create_button(|b| {
            b.custom_id("registration")
            .label("Toggle Registration")
            .style(poise::serenity_prelude::ButtonStyle::Primary)
            .disabled(reg.tournament)
          });
          row.create_button(|b| {
            b.custom_id("detail")
            .label("Detail")
            .style(poise::serenity_prelude::ButtonStyle::Primary)
          })
        })
      })
    }).await?;
    Ok(())
}

async fn getter(ctx: &Context<'_>, region: &Region) -> Result<Reg, Error> {
    let status = registration_open(ctx).await;
    let tournament_status = tournament(ctx, region).await;
    let count = count_registers(ctx, region).await?;
    Ok(Reg {
        registration: status,
        tournament: tournament_status,
        count,
        region: region.clone(),
    })
}

async fn detail(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.embed(|e| e.description("Getting player info..."))
    })
    .await?;
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let config = get_config(ctx, region).await;
    let round = find_round_from_config(&config);
    let round_number = round.parse::<i32>().unwrap_or(0);
    let collection = database.collection::<mongodb::bson::Document>(&round);
    let total = match collection.count_documents(doc! {}, None).await? as i32 {
        0 => {
            msg.edit(*ctx, |s| {
                s.embed(|e| e.description("No player found in database."))
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
    get_player_data(
        ctx,
        msg,
        region,
        first_player,
        &round_number,
        &index,
        &total,
    )
    .await?;
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
        let option = mongodb::options::FindOneOptions::builder()
            .skip(Some((index - 1) as u64))
            .build();
        let player = collection.find_one(doc! {}, option).await?.unwrap();
        get_player_data(ctx, msg, region, player, &round_number, &index, &total).await?;
    }
    Ok(())
}

async fn get_player_data(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    player: mongodb::bson::Document,
    round: &i32,
    index: &i32,
    total: &i32,
) -> Result<(), Error> {
    let name = player.get_str("name").unwrap();
    let tag = player.get_str("tag").unwrap();
    let discord_id = player.get_str("discord_id").unwrap();
    let match_id = player.get_str("match_id").unwrap_or("Not yet assigned.");
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
                ("**Match**", match_id.to_string(), true),
                ("**Battle**", battle.to_string(), true),
            ])
            .footer(|f| f.text(format!("{} out of {} players", index, total)))
        })
    })
    .await?;
    Ok(())
}

fn term(status: bool) -> String {
    if status {
        "Open".to_string()
    } else {
        "Closed".to_string()
    }
}
