use crate::{
    bracket_tournament::{config::reset_config, region::Region},
    checks::user_is_manager,
    Context, Error,
};
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use poise::ReplyHandle;
use strum::IntoEnumIterator;
///Reset tournament set up, but still keeps list of real players.
#[poise::command(slash_command)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| {
            s.reply(true)
                .content("Resetting match id, and removing mannequins and rounds...")
        })
        .await?;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Players");
        reset_match_ids(&collection, &region, &ctx, &msg).await?;
        clear_mannequins(&collection, &region, &ctx, &msg).await?;
        clear_rounds_and_reset_config(database, &region, &ctx, &msg).await?;
    }
    msg.edit(ctx, |s| s.content("Complete!")).await?;
    Ok(())
}

async fn reset_match_ids(
    collection: &Collection<Document>,
    region: &Region,
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), Error> {
    collection
        .update_many(doc! {}, doc! { "$set": { "match_id": null } }, None)
        .await?;
    msg.edit(*ctx, |s| {
        s.content(format!("All Match IDs from {} are reset!", region))
    })
    .await?;
    Ok(())
}

async fn clear_mannequins(
    collection: &Collection<Document>,
    region: &Region,
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), Error> {
    collection
        .delete_many(doc! { "name": "Mannequin" }, None)
        .await?;
    msg.edit(*ctx, |s| {
        s.content(format!("All mannequins in {} are removed!", region))
    })
    .await?;
    Ok(())
}

async fn clear_rounds_and_reset_config(
    database: &Database,
    region: &Region,
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), Error> {
    let collections = database.list_collection_names(None).await?;
    for collection in collections {
        if collection.starts_with("Round") {
            database
                .collection::<Document>(&collection)
                .drop(None)
                .await?;
        }
        if collection.starts_with("Config") {
            let config = reset_config();
            database
                .collection::<Document>(&collection)
                .update_one(doc! {}, config, None)
                .await?;
        }
    }
    msg.edit(*ctx, |s| {
        s.content(format!("All rounds in {} are removed!", region))
    })
    .await?;
    Ok(())
}
