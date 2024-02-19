use crate::{database::config::reset_config, Context, Error};
use dbc_bot::Region;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use poise::ReplyHandle;
use strum::IntoEnumIterator;

pub async fn reset(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region_option: Option<&Region>,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.content("Resetting match id, and removing mannequins and rounds...")
    })
    .await?;
    for region in Region::iter() {
        match region_option {
            Some(region_option) if region != *region_option => continue,
            _ => {}
        }
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Players");
        clear_rounds_and_reset_config(database, &region, ctx, msg).await?;
        clear_all_players(&collection).await;
    }
    msg.edit(*ctx, |s| s.content("Complete!")).await?;
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

async fn clear_all_players(collection: &Collection<Document>) {
    collection.delete_many(doc! {}, None).await.unwrap();
}
