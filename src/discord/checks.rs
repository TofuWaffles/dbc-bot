use crate::{Context, Error};
use mongodb::bson::{doc, Document};
use poise::serenity_prelude::RoleId;
use tracing::error;
use tracing::info;

pub async fn is_host(ctx: Context<'_>) -> Result<bool, Error> {
    let server_id = ctx.guild_id().unwrap().to_string();
    let doc: Document = ctx
        .data()
        .database
        .general
        .collection("Managers")
        .find_one(doc! {"server_id": server_id}, None)
        .await?
        .unwrap();
    let hosts = doc.get_array("role_id").unwrap().to_vec();
    let guild = ctx.guild_id().unwrap();
    for host in hosts.iter() {
        let id = host.as_str().unwrap().parse::<u64>()?;
        info!("Checking {id}");
        let role = RoleId::to_role_cached(RoleId(id), ctx.cache()).unwrap();
        match ctx.author().has_role(ctx.http(), guild, &role).await {
            Ok(true) => {
                return {
                    info!(
                        "{} is authenticated to host due to the role {}",
                        ctx.author().name,
                        role.name
                    );
                    Ok(true)
                }
            }
            Ok(false) => {
                continue;
            }
            Err(e) => {
                error!("{e}");
                return Ok(false);
            }
        }
    }
    info!("No permissions to host");
    Ok(false)
}

pub async fn is_mod(ctx: Context<'_>) -> Result<bool, Error> {
    let member = ctx
        .serenity_context()
        .http
        .get_member(ctx.guild_id().unwrap().into(), ctx.author().id.into())
        .await?;
    let permission = member.permissions(ctx.cache())?;

    Ok(permission.ban_members())
}
