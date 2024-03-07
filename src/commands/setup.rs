use crate::database::config::make_server_doc;
use crate::Document;
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::{doc, Bson};
use mongodb::Collection;
use poise::serenity_prelude::{MessageComponentInteraction, ReactionType};
use poise::ReplyHandle;
use std::sync::Arc;
use std::vec;
use tracing::error;

#[derive(Debug, poise::Modal)]
#[name = "Role Selection"]
struct RoleSelection{
    #[name = "Role id"]
    #[placeholder = "Please write role id here"]
    role_id: String
}



/// Setup role to interact with the  configurations of this bot
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    rename = "role-allow"
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let server_id = ctx.guild().unwrap().id.to_string();
    let collection: Collection<Document> = ctx.data().database.general.collection("Managers");
    let doc = match collection
        .find_one(doc! {"server_id": &server_id}, None)
        .await?
    {
        Some(document) => document,
        None => {
            collection
                .insert_one(
                    make_server_doc(&ctx.guild().unwrap().name, &server_id),
                    None,
                )
                .await?;
            collection
                .find_one(doc! {"server_id": &server_id}, None)
                .await?
                .unwrap()
        }
    };
    let mut hosts = match doc.get_array("role_id") {
        Ok(r) => r.clone(),
        Err(e) => {
            error!("{e}");
            let v: Vec<Bson> = vec![];
            v
        }
    };
    let msg = ctx
        .send(|s| s.reply(true).ephemeral(true).embed(|e| e.title("Setup...")))
        .await?;
    display_select_menu(&ctx, &msg, &hosts).await?;
    let mut cic = msg
        .clone()
        .into_message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120))
        .build();
    while let Some(mci) = &cic.next().await {
        mci.defer(ctx.http()).await?;

        match mci.data.custom_id.as_str() {
            "open" => {
                role_option(&ctx, &msg, mci.clone(), &collection).await?;
            }
            _ => {
                let update = doc! { "$set": { "role_id": [] } };
                collection
                    .update_one(doc! {"server_id": &server_id}, update, None)
                    .await?;
            }
        }
        hosts = collection
            .find_one(doc! {"server_id": &server_id}, None)
            .await?
            .unwrap()
            .get_array("role_id")
            .unwrap()
            .to_vec();
        display_select_menu(&ctx, &msg, &hosts).await?;
    }

    Ok(())
}

async fn display_select_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    hosts: &[Bson],
) -> Result<(), Error> {
    let accept: String = hosts
        .iter()
        .enumerate()
        .map(|(index, r)| format!("{}. <@&{}>", index + 1, r.as_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");

    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Setup").description(format!(
                "
Following roles can access Host menu:\n{accept}"
            ))
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("open")
                        .label("Add role")
                        .style(poise::serenity_prelude::ButtonStyle::Success)
                        .emoji(ReactionType::Unicode("👮".to_string()))
                })
            })
            .create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("reset")
                        .label("Reset")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("🔴".to_string()))
                })
            })
        })
    })
    .await?;
    Ok(())
}


async fn role_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    mci: Arc<MessageComponentInteraction>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    let server_id = ctx.guild().unwrap().id.to_string();
    match poise::execute_modal_on_component_interaction::<RoleSelection>(ctx, mci, None, None)
        .await?
    {
        Some(r) => {
                let update = doc! { "$push": { "role_id": &r.role_id } };
                collection
                    .update_one(doc! {"server_id": &server_id}, update, None)
                    .await?;
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("The role has been set!").description(format!(
                        "The role **<@&{}>** has been selected for this tournament!
                        Directing back to selecting role menu...", r.role_id
                        
                    ))
                })
            })
            .await?;
        }
        None => {
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("Failed to add more role!").description(format!(
                        "No role has been selected! Please try again!"
                        
                    ))
                })
            })
            .await?;
        }
    }
    Ok(())
}