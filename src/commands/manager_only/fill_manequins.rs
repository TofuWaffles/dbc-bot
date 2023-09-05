use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};

use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use strum::IntoEnumIterator;
/// Fill database with mannequins for testing purpose
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn fill_mannequins(
    ctx: Context<'_>,
    #[description = "Put your tag here (without #)"] quantity: i32,
) -> Result<(), Error> {
    ctx.say("Filling databases with mannequins...").await?;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let collection: Collection<Document> = database.collection("Player");
        for _ in 0..quantity {
            collection
                .insert_one(add_mannequin(&region, None, None), None)
                .await?;
        }
        ctx.channel_id()
            .send_message(ctx, |s| {
                s.content(format!("Added {} mannequins to {}", quantity, region))
            })
            .await?;
    }
    ctx.channel_id()
        .send_message(ctx, |s| s.content("Complete!"))
        .await?;
    Ok(())
}
