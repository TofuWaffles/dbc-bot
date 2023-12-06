use crate::Document;
use crate::Region;
use crate::{Context, Error};
use mongodb::Collection;
use tracing::error;

pub async fn add_player(
    ctx: &Context<'_>,
    player: &Option<Document>,
    region: &Option<Region>,
) -> Result<(), Error> {
    let collection: Collection<Document> =
        ctx.data().database.regional_databases[&region.clone().unwrap()].collection("Players");
    match collection.insert_one(player.clone().unwrap(), None).await {
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
