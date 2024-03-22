use crate::database::battle::force_lose;
use crate::database::config::get_config;
use crate::database::find::{
    find_enemy_by_match_id_and_self_tag, find_enemy_of_mannequin, find_player_by_discord_id,
    find_round_from_config,
};
use crate::database::remove::remove_player;
use crate::database::update::{set_ready, update_result};
use crate::discord::log::{Log, LogType};
use crate::discord::prompt::prompt;
use crate::discord::role::remove_role;
use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use mongodb::bson::{doc, Document};
use mongodb::{Collection, Cursor};
use poise::serenity_prelude::UserId;
use poise::ReplyHandle;
use std::sync::Arc;
use std::vec;
use tracing::{error, info};
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

    #[name = "Proof (URLs - separated by space)"]
    #[placeholder = "Images as links. If the attachments are screenshot, leave blank and use /log-update after."]
    proof: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct Form {
    pub user_id: String,
    pub reason: String,
    pub proof: String,
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
            .description(r#"Please enter the user ID of the player you want to disqualify.
See [this guide](https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-) for more information.
If you have URLs of the evidence image, you can paste it in Proof field.
"#)
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
                    proof: data.proof.unwrap_or("".to_string()),
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
            let info = format!(
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
            );
            let user = UserId(form.user_id.parse::<u64>().unwrap())
                .to_user(ctx.http())
                .await?;
            let more_info = if remove_role(ctx, &user, region).await.is_err() {
                "⚠️ The user is removed from the tournament, but it is unable to remove the role from this player! The player may not be present in the server."
            } else {
                "<:tick:1187839626338111600> Removed the player successfully!"
            };
            prompt(
                ctx,
                msg,
                "Successfully remove player!",
                format!("{}\n{}", info, more_info),
                None,
                Some(0x50C878),
            )
            .await?;
            set_ready(
                ctx,
                region,
                round,
                enemy.unwrap().get_str("discord_id").unwrap_or("0"),
            )
            .await?;
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
    _false_battles: &mut Cursor<Document>,
) -> Result<(), Error> {
    let collection: Collection<Document> = ctx
        .data()
        .database
        .regional_databases
        .get(region)
        .unwrap()
        .collection(&round);
    let mut battles_handle = collection
        .find(doc! {"battle": false, "ready": false}, None)
        .await?;
    let counts = collection
        .count_documents(doc! {"battle": false, "ready": false}, None)
        .await? as usize;
    info!("Counts: {}", counts);
    prompt(
        ctx,
        msg,
        "Mass disqualification",
        format!(
            r#"There are {} players that are determined to be inactive.
<a:loading:1187839622680690689>  Disqualifying inactive players...
            "#,
            counts
        ),
        None,
        Some(0x50C878),
    )
    .await?;
    let log = Log::new(ctx, region, LogType::DisqualifyInactives).await?;
    let mut players = vec![];
    let perc = counts / 10;
    let mut index = 0;
    while let Some(player) = battles_handle.next().await {
        match player {
            Ok(player) => match player.get_str("discord_id") {
                Ok(id) => {
                    match player.get_bool("ready") {
                        Ok(false) => {
                            if force_lose(ctx, region, &player).await.is_ok() {
                                players.push(format!("<@{id}>"));
                            }
                        }

                        _ => {
                            continue; //Manually handled
                        }
                    }
                }
                Err(_) => {
                    // Mannequins
                    let match_id = player.get_i32("match_id")?;
                    if let Some(opponent) =
                        find_enemy_of_mannequin(ctx, region, round, &match_id).await
                    {
                        match update_result(ctx, region, round, &opponent, &player).await {
                            Err(e) => {
                                error!("{e}");
                            }
                            Ok(_) => {}
                        };
                    }
                }
            },
            Err(_) => {
                continue;
            }
        }
        if index % perc == 0 {
            let progress = index / perc;
            prompt(
                ctx,
                msg,
                "Mass disqualification",
                format!(
                    r#"Disqualifying inactive players...
<a:loading:1187839622680690689> {}% completed...
Progress bar: {} 
                    "#,
                    progress * 10,
                    format!(
                        "{done}{yet}",
                        done = "█".repeat(progress),
                        yet = "░".repeat(10 - progress)
                    ),
                ),
                None,
                Some(0x50C878),
            )
            .await?;
        }
        index += 1;
    }
    let m = log.disqualify_inactive_logs(players).await?;
    prompt(
        ctx,
        msg,
        "Mass disqualification",
        format!(
            "<:tick:1187839626338111600> Successfully disqualified all inactive players. Check [log]({link}) for more details.",
            link = m.link()
        ),
        None,
        Some(0x50C878),
    )
    .await
}
