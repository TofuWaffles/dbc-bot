use crate::players::submit::submit_result;
use crate::players::view2::{view_managers, view_opponent};
use dbc_bot::Region;

use super::prompt::prompt;
use crate::host::config::configurate;
use crate::host::disqualify::disqualify_players;
use crate::players::register::register_menu;
use crate::{
    players::{deregister::deregister_menu, view::view_info},
    Context, Error,
};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::{
    serenity_prelude::{ButtonStyle, ReactionType},
    ReplyHandle,
};

const TIMEOUT: u64 = 300;
/// Displays a registration menu with various options.
/// - `ctx`: Context<'_>.
/// - `msg`: The message to edit.
/// - `register`: Whether to show the register button.
/// - `view`: Whether to show the view button.
/// - `deregister`: Whether to show the deregister button.
/// - `help`: Whether to show the help button.
/// - `player`: The player document.
pub async fn registration_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    register: bool,
    view: bool,
    deregister: bool,
    help: bool,
    player: Option<Document>,
) -> Result<(), Error> {
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("register")
                        .label("Register")
                        .disabled(!register)
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("📝".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("personal")
                        .label("View\nPersonal\nInfo")
                        .disabled(!view)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🤓".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("deregister")
                        .label("Deregister")
                        .disabled(!deregister)
                        .style(ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🚪".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("help")
                        .label("Help")
                        .disabled(!help)
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("❓".to_string()))
                })
            })
        })
        .embed(|e| {
            e.title("Registration Menu")
                .description("Below are the available options!")
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "register" => {
                mci.defer(&ctx.http()).await?;
                return register_menu(ctx, msg).await;
            }
            "deregister" => {
                mci.defer(&ctx.http()).await?;
                return deregister_menu(ctx, msg, player.unwrap()).await;
            }
            "personal" => {
                mci.defer(&ctx.http()).await?;
                return view_info(ctx, msg, player.unwrap()).await;
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                  ctx,
                  msg,
                  "This is still under development!", 
                  "This feature is still under development, please be patient!", 
                  Some("https://tenor.com/view/josh-hutcherson-josh-hutcherson-whistle-edit-whistle-2014-meme-gif-1242113167680346055"),
                  None
              ).await;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn tournament_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    schedule: bool,
    managers: bool,
    submit: bool,
    help: bool,
    player: Option<Document>,
) -> Result<(), Error> {
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("enemy")
                        .label("View\nOpponent")
                        .disabled(!schedule)
                        .style(ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🤓".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("managers")
                        .label("View\nManagers")
                        .disabled(!managers)
                        .style(ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🛡️".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("submit")
                        .label("Submit\nResults")
                        .disabled(!submit)
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("⚔️".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("help")
                        .label("Help")
                        .disabled(!help)
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("❓".to_string()))
                })
            })
        })
        .embed(|e| {
            e.title("Tournament Menu")
                .description("Below are the available options!")
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "enemy" => {
                mci.defer(&ctx.http()).await?;
                return view_opponent(ctx).await;
            }
            "managers" => {
                mci.defer(&ctx.http()).await?;
                return view_managers(ctx).await;
            }
            "submit" => {
                mci.defer(&ctx.http()).await?;
                return submit_result(ctx).await;
            }
            "view" => {
                mci.defer(&ctx.http()).await?;
                return todo!();
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                  ctx,
                  msg,
                  "This is still under development!", 
                  "This feature is still under development, please be patient!", 
                  Some("https://tenor.com/view/josh-hutcherson-josh-hutcherson-whistle-edit-whistle-2014-meme-gif-1242113167680346055"),
                  None
              ).await;
            }
            _ => {}
        }
    }
    Ok(())
}

pub async fn mod_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    disqualify: bool,
    managers: bool,
    submit: bool,
    help: bool,
) -> Result<(), Error> {
    msg.edit(*ctx, |e| {
        e.components(|c| {
            c.create_action_row(|r| {
                r.create_button(|b| {
                    b.custom_id("disqualify")
                        .label("Disqualify\nPlayer")
                        .disabled(!disqualify)
                        .style(ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🔨".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("registration")
                        .label("Registration\nOptions")
                        .disabled(!managers)
                        .style(ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("📥".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("configuration")
                        .label("Configuration\nROptions")
                        .disabled(!submit)
                        .style(ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("⚙️".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("help")
                        .label("Help")
                        .disabled(!help)
                        .style(ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("❓".to_string()))
                })
            })
        })
        .embed(|e| {
            e.title("Host-only menu").description(format!(
                "The following mod menu is set for region: {}",
                region
            ))
        })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "disqualify" => {
                mci.defer(&ctx.http()).await?;
                return disqualify_players(ctx, msg).await;
            }
            "registration" => {
                mci.defer(&ctx.http()).await?;
                return todo!();
            }
            "configuration" => {
                mci.defer(&ctx.http()).await?;
                return configurate(ctx, msg, region).await;
            }
            "help" => {
                mci.defer(&ctx.http()).await?;
                return prompt(
                  ctx,
                  msg,
                  "This is still under development!", 
                  "This feature is still under development, please be patient!", 
                  Some("https://tenor.com/view/josh-hutcherson-josh-hutcherson-whistle-edit-whistle-2014-meme-gif-1242113167680346055"),
                  None
              ).await;
            }
            _ => {}
        }
    }
    Ok(())
}
