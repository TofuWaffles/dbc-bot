use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_player_by_discord_id, find_round_from_config,
};
use crate::database::remove::remove_player;
use crate::database::update::{update_match_id, update_result};
use crate::discord::log::{Log, LogType};
use crate::discord::prompt::{self, prompt};
use crate::discord::role::remove_role;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::Document;
use mongodb::{Collection, Cursor};
use poise::serenity_prelude::UserId;
use poise::ReplyHandle;
use std::sync::Arc;
use std::vec;
use tracing::error;
const TIMEOUT: u64 = 120;
#[derive(Debug, poise::Modal)]
#[name = "Disqualify Modal"]
struct DisqualifyModal {
    #[name = "User Id to be disqualified:"]
    #[placeholder = "Make sure the user ID is provided, not the username"]
    user_id: String,

    #[name = "Reason"]
    #[placeholder = "Custom reason or leave blank for default reason"]
    reason: String,
}

#[derive(Debug, Default, Clone)]
pub struct Form {
    pub user_id: String,
    pub reason: String,
}

pub async fn disqualify_players(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("🔨 Disqualify Players")
                .description("Please enter the user ID of the player you want to disqualify.")
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| b.custom_id("open_modal").label("Disqualify Player"))
            })
        })
    })
    .await?;
    let mut form = Form::default();
    let mut player = Document::new();
    let round = find_round_from_config(&get_config(ctx, region).await);

    disqualify_id(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "open_modal" => {
                form = create_disqualify_modal(ctx, mci.clone()).await?;
                match find_player_by_discord_id(
                    ctx,
                    region,
                    form.user_id.parse::<u64>().unwrap_or(0),
                    &round,
                )
                .await
                {
                    Ok(Some(p)) => {
                        player = p;
                        display_confirmation(ctx, msg, &player).await?
                    }
                    Ok(None) => {
                        return prompt(
                            ctx,
                            msg,
                            "Not found",
                            "No player found with the given user ID",
                            None,
                            Some(0xFF0000),
                        )
                        .await;
                    }
                    Err(e) => {
                        error!("{e}");
                        return prompt(
                            ctx,
                            msg,
                            "ERROR",
                            "Unable to find the player",
                            None,
                            Some(0xFF0000),
                        )
                        .await;
                    }
                }
            }
            "confirm" => return post_confirm(ctx, msg, &player, region, &mut form, &round).await,

            "cancel" => {
                mci.defer(&ctx.http()).await?;
                prompt(
                    ctx,
                    msg,
                    "Player disqualification cancelled",
                    "You can return to this menu by running </index:1181542953542488205>",
                    None,
                    None,
                )
                .await?;
            }
            _ => {
                continue;
            }
        }
    }
    Ok(())
}

async fn disqualify_id(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b|{
        b.embed(|e|{
            e.title("🔨 Disqualify Players - Step 1: Enter the user ID")
            .description("Please enter the user ID of the player you want to disqualify. See [this guide](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-) for more information.")
        })
        .components(|c|{
            c.create_action_row(|a|{
                a.create_button(|b|{
                    b.custom_id("open_modal")
                    .label("Disqualify Player")
                })
        })
    })}).await?;
    Ok(())
}

async fn display_confirmation(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: &Document,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Confirm")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .custom_id("confirm")
                })
                .create_button(|b| {
                    b.label("Cancel")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .custom_id("cancel")
                })
            })
        })
        .embed(|e| {
            e.author(|a| a.name(ctx.author().name.clone()))
                .title("🔨 Disqualify Players - Step 2: User confirmation")
                .description(
                    "**Please confirm this is the player that you would like to disqualify.**",
                )
                .fields(vec![
                    (
                        "Mention",
                        format!("<@{}>", player.get_str("discord_id").unwrap()),
                        true,
                    ),
                    (
                        "Region",
                        player.get_str("region").unwrap().to_string(),
                        true,
                    ),
                    ("Name", player.get_str("name").unwrap().to_string(), true),
                    ("Tag", player.get_str("tag").unwrap().to_string(), true),
                ])
                .timestamp(ctx.created_at())
        })
    })
    .await?;

    Ok(())
}

async fn create_disqualify_modal(
    ctx: &Context<'_>,
    mci: Arc<poise::serenity_prelude::MessageComponentInteraction>,
) -> Result<Form, Error> {
    loop {
        let result = poise::execute_modal_on_component_interaction::<DisqualifyModal>(
            ctx,
            mci.clone(),
            None,
            None,
        )
        .await?;
        match result {
            Some(data) => {
                return Ok(Form {
                    user_id: data.user_id,
                    reason: data.reason,
                });
            }
            None => continue,
        }
    }
}

async fn post_confirm(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: &Document,
    region: &Region,
    form: &mut Form,
    round: &str,
) -> Result<(), Error> {
    match remove_player(ctx, player, region).await {
        Ok(_) => {
            let log = Log::new(ctx, region, LogType::Disqualify).await?;
            let log_msg = log.send_disqualify_log(form, round).await?;
            let enemy = find_enemy_by_match_id_and_self_tag(
                ctx,
                region,
                round,
                &player.get_i32("match_id").unwrap_or(0),
                player.get_str("tag").unwrap_or(""),
            )
            .await;
            prompt(
                ctx,
                msg,
                "Successfully remove player!",
                format!(
                    r#"The log has been recorded at [here]({link}).
Please remind this player's opponent!
Round: {round}
Match: {match_id}   
Opponent: 
- In-game name: {enemy_name}
- In-game tag: {enemy_tag}
- Discord name: {enemy_discord}
- Discord id: {enemy_discord_id}             
"#,
                    link = log_msg.link(),
                    match_id = player.get_i32("match_id").unwrap_or(0),
                    round = round,
                    enemy_name = enemy
                        .as_ref()
                        .map_or_else(|| "Not found", |e| e.get_str("name").unwrap_or("")),
                    enemy_tag = enemy
                        .as_ref()
                        .map_or_else(|| "Not found", |e| e.get_str("tag").unwrap_or("")),
                    enemy_discord = enemy
                        .as_ref()
                        .map_or_else(|| "Not found", |e| e.get_str("discord_name").unwrap_or("")),
                    enemy_discord_id = enemy.as_ref().map_or_else(
                        || "Not found",
                        |e| e.get_str("discord_id").unwrap_or("Not found")
                    )
                ),
                None,
                Some(0x50C878),
            )
            .await?;
            let user = UserId(form.user_id.parse::<u64>().unwrap())
                .to_user(ctx.http())
                .await?;
            match remove_role(ctx, &user, region).await {
                Ok(_) => {}
                Err(e) => {
                    error!("{e}");
                    return prompt(ctx, msg, "Failed to remove the role", "The user is removed from the tournament, but it is unable to remove the role from this player!", None, Some(0xFF0000)).await;
                }
            }
        }
        Err(e) => {
            error!("{e}");
            return prompt(
                ctx,
                msg,
                "ERROR",
                "Unable to remove the player!",
                None,
                Some(0xFF0000),
            )
            .await;
        }
    };
    Ok(())
}

pub async fn mass_disqualify_wrapper(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    round: &str,
    false_battles: &mut Cursor<Document>,
) -> Result<(), Error> {
    let log = Log::new(ctx, region, LogType::DisqualifyInactives).await?;
    let mut players = vec![];
    while let Some(player) = false_battles.next().await {
        match player {
            Ok(player) => match player.get_bool("ready") {
                Ok(false) => {
                    if disqualify_unready(ctx, region, round, &player).await.is_ok(){
                        players.push(format!("<@{}>", player.get_str("discord_id").unwrap_or("0")));    
                    }
                }
                _ => {
                    continue;
                }
            },
            Err(_) => {
                continue;
            }
        }
    }
    let m = log.disqualify_inactive_logs(players).await?;
    return prompt(
        ctx,
        msg,
        "Mass disqualification",
        format!(
            "Successfully disqualified all inactive players. Check [log]({link}) for more details.", link=m.link()
        ),
        None,
        Some(0x50C878),
    ).await;
}

async fn disqualify_unready(
    ctx: &Context<'_>,
    region: &Region,
    round: &str,
    player: &Document,
) -> Result<(), Error> {
    let match_id = player.get_i32("match_id").unwrap_or(0);
    let player_tag = player.get_str("tag").unwrap_or("");
    if let Some(enemy) =
        find_enemy_by_match_id_and_self_tag(ctx, region, round, &match_id, player_tag).await
    {
        if enemy.get_bool("ready").unwrap_or(false) {
            let database = ctx.data().database.regional_databases.get(region).unwrap();
            let round_coll = database.collection(round);
            let next_round: Collection<Document> = database.collection(
                format!(
                    "Round {}",
                    round
                        .split_whitespace()
                        .nth(1)
                        .unwrap()
                        .parse::<i32>()
                        .unwrap()
                        + 1
                )
                .as_str(),
            );
            next_round
                .insert_one(update_match_id(enemy.clone()), None)
                .await?;
            let defeated_user = UserId(player.get_str("discord_id").unwrap_or("0").parse::<u64>()?)
                .to_user(ctx.http())
                .await?;
            if let Err(e) = remove_role(ctx, &defeated_user, &region).await {
                error!("{e}");
            }
            update_result(&round_coll, &enemy, &player).await?;
        }
    }
    Ok(())
}
