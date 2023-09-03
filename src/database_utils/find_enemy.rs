extern crate mongodb;
use crate::bracket_tournament::region::Region;
use crate::Context;
use mongodb::{
    bson::{doc, Bson, Document},
    Collection,
};

pub async fn find_enemy(
    ctx: &Context<'_>,
    region: &Region,
    match_id: &i32,
    other_tag: &String,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection: Collection<Document> = database.collection("Player");
    let filter = doc! {
        "match_id": match_id,
        "tag": {
           " $ne": other_tag
        }
    };
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => 
            match enemy.get("tag").unwrap() == &Bson::Null {
                true => {
                    //Bye players
                    ctx.send(|s| {
                        s.reply(true)
                        .ephemeral(true)
                        .embed(|e|{
                            e.title("**Bye! See you... next round!**")
                            .description("You are the lucky player to receive a bye for this round!")
                        })    
                    }).await.unwrap();
                    None
                }
                false => Some(enemy),
            },
        Ok(None) => None,
        Err(err) => {
            eprintln!("Error while querying database: {:?}", err);
            None
        }
    }
}
