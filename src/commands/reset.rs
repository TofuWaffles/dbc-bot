use crate::bracket_tournament::region::Region;
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;
use tracing::{info, instrument};
///Reset all match_id of players and remove mannequins
#[instrument]
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    info!("Attempting to reset the tournament(s)");
    ctx.say("Resetting match id, and removing mannequins and rounds...")
        .await?;
    for region in Region::iter() {
        info!("Resetting for tournament region {}", region);
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
        }
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("All rounds in {} are removed!", region))
            })
            .await?;
    }
    info!("Finished resetting tournament(s)");
    ctx.channel_id()
        .send_message(ctx, |s| s.content("Complete!"))
        .await?;
    Ok(())
}
