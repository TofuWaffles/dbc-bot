use crate::misc::Region;
use crate::{Context, Error};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    options::AggregateOptions,
    Collection,
};
use strum::IntoEnumIterator;

///Run this command once registration closes to start the tournament.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn start_tournament(ctx: Context<'_>) -> Result<(), Error> {
    let collection: Collection<Document> = ctx.data().database.collection("Player");

    // Update registration IDs for each document
    let mut player_cursor = collection.find(None, None).await?;
    let mut count = 1;

    while let Ok(Some(mut document)) = player_cursor.try_next().await {
        let registration_id = count;
        count += 1;

        document.insert("registration_id", registration_id);

        collection.replace_one(
            doc! { "_id": document.get_object_id("_id")? },
            document,
            None,
        ).await?;
    }

    // Filter out by region and put them into a new collection
    for region in Region::iter() {
        let pipeline: Vec<Document> = vec![
            doc! {
                "$match": {
                    "region": format!("{:?}", region),
                }
            },
            doc! {
                "$out": format!("{:?}_Registration", region),
            },
        ];

        collection
            .aggregate(
                pipeline,
                AggregateOptions::builder().allow_disk_use(true).build(),
            )
            .await?;
    }
    ctx.say("Response here to not fail interaction").await?;
    Ok(())
}
