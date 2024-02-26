use crate::{discord::menu::mod_menu, Context, Error};
use dbc_bot::Region;
use mongodb::bson::Document;
use poise::serenity_prelude::RoleId;
use tracing::{error, info};

/// Host all-in-one command
#[poise::command(slash_command, guild_only)]
pub async fn host(
    ctx: Context<'_>,
    #[description = "Select region to configurate"] region: Region,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match is_host(ctx).await {
        Ok(true) => {}
        Ok(false) => {
            ctx.send(|s| {
                s.ephemeral(true)
                    .reply(true)
                    .content("You don't have permissions to host")
            })
            .await?;
            return Ok(());
        }
        Err(e) => {
            error!("{e}");
            return Ok(());
        }
    }
    let msg = ctx
        .send(|s| {
            s.embed(|e| {
                e.title("Host-only menu")
                    .description(format!(
                        "The following mod menu is set for region: {region}"
                    ))
                    .image("")
            })
        })
        .await?;
    mod_menu(&ctx, &msg, &region, true, true, true, true).await
}

async fn is_host(ctx: Context<'_>) -> Result<bool, Error> {
    let doc: Document = ctx
        .data()
        .database
        .general
        .collection("Managers")
        .find_one(None, None)
        .await?
        .unwrap();
    let hosts = doc.get_array("role_id").unwrap().to_vec();
    let guild = ctx.guild_id().unwrap();
    for host in hosts.iter() {
        let id = host.as_str().unwrap().parse::<u64>()?;
        let role = RoleId::to_role_cached(RoleId(id), ctx.cache()).unwrap();
        match ctx.author().has_role(ctx.http(), guild, role).await {
            Ok(true) => return Ok(true),
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
