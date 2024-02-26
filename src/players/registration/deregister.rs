use crate::database::config::get_config;
use crate::database::remove::remove_registration;
use crate::discord::prompt;
use crate::players::registration::deregister::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::{CustomError, Region};
use futures::StreamExt;
use mongodb::bson::Document;
use poise::serenity_prelude::ButtonStyle;
use poise::ReplyHandle;
use std::ops::Deref;
use tracing::info;
const REGISTRATION_TIME: u64 = 120;
async fn display_deregister_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |b| {
        b.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Deregister")
                        .style(ButtonStyle::Danger)
                        .custom_id("deregister")
                })
            })
        })
        .embed(|e| {
            e.title("Deregisteration")
                .description("Are you sure you want to deregister from the tournament?".to_string())
        })
    })
    .await?;
    Ok(())
}

pub async fn deregister_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    display_deregister_menu(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(REGISTRATION_TIME));
    let mut cic = cib.build();
    if let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "deregister" => {
                let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
                remove_registration(ctx, &player).await?;
                remove_role(ctx, msg, &get_config(ctx, &region).await).await?;
                msg.edit(*ctx, |b| {
                    b.components(|c| c)
                        .embed(|e| {
                            e.title("Deregistration").description(
                                "You have been deregistered from the tournament\nYou can safely dismiss this message\nOr you can run </index:1181542953542488205> again."
                            )
                        })
                })
                .await?;
            }
            _ => {
                unreachable!("This should never happen!")
            }
        }
    }

    Ok(())
}

async fn remove_role(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    config: &Document,
) -> Result<(), Error> {
    let role_id = config.get_str("role").unwrap().parse::<u64>().unwrap();
    let mut member = match ctx.author_member().await {
        Some(m) => m.deref().to_owned(),
        None => {
            let user = *ctx.author().id.as_u64();
            prompt(
                ctx,
                msg,
                "Failed to get member",
                format!("Failed to get member for <@{}>", user),
                None,
                Some(0xFF0000),
            )
            .await?;
            info!("Failed to assign role for <@{}>", user);
            return Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{user}>",
            ))));
        }
    };
    match member.remove_role((*ctx).http(), role_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let user = *ctx.author().id.as_u64();
            prompt(
                ctx,
                msg,
                "Failed to remove the regional role",
                format!("Failed to remove the regional role for <@{}>", user),
                None,
                Some(0xFF0000),
            )
            .await
        }
    }
}
