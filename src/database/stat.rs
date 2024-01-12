use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::bson::doc;

pub async fn count_registers(ctx: &Context<'_>, region: &Region) -> Result<i32, Error> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let count: i32 = match database
        .collection::<i32>("Players")
        .count_documents(doc! { "name": { "$ne": "Mannequin" } }, None)
        .await
    {
        Ok(result) => result.try_into().unwrap_or_else(|_err| {
            let drop = ctx.say("There are way too many documents, so it is unable to convert from u64 to i32 due to overflow.");
            std::mem::drop(drop);
            0
        }),
        Err(err) => {
            let drop = ctx.say(format!("Error: {}", err));
            std::mem::drop(drop);
            0
        }
    };
    Ok(count)
}
