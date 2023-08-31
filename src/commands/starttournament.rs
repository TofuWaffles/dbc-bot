use crate::misc::Region;
use crate::{Context, Error};
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
