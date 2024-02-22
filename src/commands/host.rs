use crate::{discord::menu::mod_menu, Context, Error};
use dbc_bot::Region;
use mongodb::bson::Document;
use poise::serenity_prelude::RoleId;

// Host all-in-one command
#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn host(
    ctx: Context<'_>,
    #[description = "Select region to configurate"] region: Region,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
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
    let host = doc.get_str("role_id").unwrap();
    let guild = ctx.guild_id().unwrap();
    let role = RoleId::to_role_cached(RoleId(host.parse::<u64>()?), ctx.cache()).unwrap();
    Ok(ctx.author().has_role(ctx.http(), guild, role).await?)
}
