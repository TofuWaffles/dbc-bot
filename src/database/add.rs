use crate::Document;
use crate::Region;
use crate::{Context, Error};
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use mongodb::Collection;
use tracing::error;

use super::mannequin::add_mannequin;

pub async fn add_player(
    ctx: &Context<'_>,
    player: Document,
    region: &Region,
) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Players");
    let filter = doc! { "discord_id": ctx.author().id.to_string()};
    let options = UpdateOptions::builder().upsert(true).build();
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


pub async fn insert_mannequins(ctx: &Context<'_>, region: &Region, byes: i32) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Players");
    match byes {
        0 => {}
        _ => {
            for _ in 1..=byes {
                let mannequin = add_mannequin(region, None);
                collection.insert_one(mannequin, None).await?;
            }
        }
    }
    Ok(())
}