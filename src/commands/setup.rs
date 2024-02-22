use crate::database::config::make_server_doc;
use crate::Document;
use crate::{Context, Error};
use futures::StreamExt;
use mongodb::bson::doc;
use poise::{
    serenity_prelude::{CreateSelectMenuOption, Role, RoleId},
    ReplyHandle,
};
use std::collections::HashMap;
use tracing::error;

/// Setup role to interact with the  configurations of this bot
#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES")]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let roles = ctx.guild_id().unwrap().roles(ctx.http()).await?;
    let server_id = ctx.guild().unwrap().id.to_string();
    let collection = ctx.data().database.general.collection("Managers");
    match collection
        .find_one(doc! {"server_id": &server_id}, None)
        .await?
    {
        Some(_) => {
            ctx.send(|s| {
                s.reply(true)
                    .content("This server has already been set up!")
                    .ephemeral(true)
            })
            .await?;
            return Ok(());
        }
        None => {}
    }
    collection
        .insert_one(
            make_server_doc(&ctx.guild().unwrap().name, &server_id),
            None,
        )
        .await?;
    let doc: Document = collection.find_one(None, None).await?.unwrap();
    let host = match doc.get_array("role_id") {
        Ok(r) => {
            let mut res = None;
            for (id, role) in roles.iter() {
                if r.iter()
                    .any(|item| item.as_str() == Some(id.to_string().as_str()))
                {
                    res = Some(role);
                    break;
                }
            }
            res
        }
        Err(e) => {
            error!("{}", e); // Fix interpolation
            None
        }
    };
    let msg = ctx
        .send(|s| s.reply(true).ephemeral(true).embed(|e| e.title("Setup...")))
        .await?;
    display_select_menu(&ctx, &msg, &roles, host).await?;
    let mut cic = msg
        .clone()
        .into_message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120))
        .build();
    while let Some(mci) = &cic.next().await {
        let role = mci.data.values[0].as_str();
        let update = doc! { "$push": { "role_id": role } };
        collection.update_one(doc.clone(), update, None).await?;
    }

    Ok(())
}

async fn display_select_menu(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    roles: &HashMap<RoleId, Role>,
    host: Option<&Role>,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Setup").description(format!(
                r#"
Please pick the role for hosts!
Host: {}
"#,
                host.map_or_else(
                    || "Not yet assigned".to_string(),
                    |r| format!("<@&{}>", r.id)
                )
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
        })
    })
    .await?;
    Ok(())
}
