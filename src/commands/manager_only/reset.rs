use crate::{
    bracket_tournament::{config::reset_config, region::Region},
    Context, Error, checks::user_is_manager,
};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;
///Reset tournament set up, but still keeps list of real players.
#[poise::command(
    slash_command,
)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    if !user_is_manager(ctx).await? { return Ok(()) }

    ctx.say("Resetting match id, and removing mannequins and rounds...")
        .await?;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Player");
        collection
            .update_many(doc! {}, doc! { "$set": { "match_id": null } }, None)
            .await?;
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All Match IDs from {} are reset!", region))
            })
            .await?;
        collection
            .delete_many(doc! { "name": "Mannequin" }, None)
            .await?;
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All mannequins in {} are removed!", region))
            })
            .await?;
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
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All rounds in {} are removed!", region))
            })
            .await?;
    }
    ctx.channel_id()
        .send_message(ctx, |s| s.content("Complete!"))
        .await?;
    Ok(())
}
