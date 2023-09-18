use mongodb::bson::Document;
use strum::IntoEnumIterator;

use crate::bracket_tournament::config::{
    disable_registration_config, enable_registration_config, get_config,
};
use crate::bracket_tournament::region::Region;
use crate::checks::user_is_manager;
use crate::{Context, Error};
/// Open registration for a specific region or all.
#[poise::command(slash_command, guild_only, rename = "open-registration")]
pub async fn open_registration(
    ctx: Context<'_>,
    #[description = "(Optional) Open registration for specific region, or all"]
    region_option: Option<Region>,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Enabling registration...")
        })
        .await?;
    for region in Region::iter() {
        match region_option {
            Some(ref region_option) if region != *region_option => continue,
            _ => {}
        }
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection = database.collection::<Document>("Config");
        let config = get_config(database).await;
        collection
            .update_one(config, enable_registration_config(), None)
            .await?;
        msg.edit(ctx, |s| {
            s.content(format!("Registration is now open for {}!", region))
                .ephemeral(true)
                .reply(true)
        })
        .await?;
    }
    Ok(())
}

/// Close registration for a specific region or all.
#[poise::command(slash_command, guild_only, rename = "close-registration")]
pub async fn close_registration(
    ctx: Context<'_>,
    #[description = "(Optional) Close registration for specific region, or all"]
    region_option: Option<Region>,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Closing registration...")
        })
        .await?;
    for region in Region::iter() {
        match region_option {
            Some(ref region_option) if region != *region_option => continue,
            _ => {}
        }
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection = database.collection::<Document>("Config");
        let config = get_config(database).await;
        collection
            .update_one(config, disable_registration_config(), None)
            .await?;
        msg.edit(ctx,|s| {
            s.content(format!("Registration is now closed for {}!", region))
                .ephemeral(true)
                .reply(true)
        })
        .await?;
    }
    Ok(())
}
