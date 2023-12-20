use crate::Document;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use mongodb::Collection;
use tracing::error;

pub async fn add_player(
    ctx: &Context<'_>,
    player: Document,
    region: &Option<Region>,
) -> Result<(), Error> {
    let filter = doc! { "discord_id": ctx.author().id.to_string()};
    let options = UpdateOptions::builder().upsert(true).build();
    let collection: Collection<Document> =
        ctx.data().database.regional_databases[&region.clone().unwrap()].collection("Players");
    let update = doc! {
        "$set": player
    };
    match collection.update_one(filter, update, options).await {
        Ok(_) => {}
        Err(err) => match err.kind.as_ref() {
            mongodb::error::ErrorKind::Command(code) => {
                error!("Command error: {:?}", code);
            }
            mongodb::error::ErrorKind::Write(code) => {
                error!("Write error: {:?}", code);
            }
            _ => {
                error!("Error: {:?}", err);
            }
        },
    };
    Ok(())
}
