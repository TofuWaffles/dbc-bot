use crate::database::config::make_server_doc;
use crate::Document;
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::{doc, Bson};
use mongodb::Collection;
use poise::serenity_prelude::ReactionType;
use poise::{
    serenity_prelude::{CreateSelectMenuOption, Role, RoleId},
    ReplyHandle,
};
use std::collections::HashMap;
use std::vec;
use tracing::error;

/// Setup role to interact with the  configurations of this bot
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES",
    aliases("role-allow")
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let roles = ctx.guild_id().unwrap().roles(ctx.http()).await?;
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
    display_select_menu(&ctx, &msg, &roles, &hosts).await?;
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
            "setup" => {
                let role = mci.data.values[0].as_str();
                let update = doc! { "$push": { "role_id": role } };
                collection
                    .update_one(doc! {"server_id": &server_id}, update, None)
                    .await?;
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
        display_select_menu(&ctx, &msg, &roles, &hosts).await?;
    }

    Ok(())
}

async fn display_select_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    roles: &HashMap<RoleId, Role>,
    hosts: &Vec<Bson>,
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
            c.create_action_row(|c| {
                c.create_select_menu(|s| {
                    s.custom_id("setup")
                        .placeholder("Select a host role to access the bot's configuration.")
                        .options(|o| {
                            for (role_id, role) in roles.iter() {
                                let mut option = CreateSelectMenuOption::default();
                                option.label(role.clone().name).value(role_id.to_string());
                                o.add_option(option);
                            }
                            o
                        })
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
