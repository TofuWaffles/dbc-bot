use crate::{
    bracket_tournament::region::Region,
    Context
};
use mongodb::{
    bson::{
        doc,
        Bson,
        Document},
    Collection,
};

pub async fn find_enemy(
    ctx: &Context<'_>,
    region: &Region,
    round: &i32,
    match_id: &i32,
    other_tag: &String,
) -> Option<Document> {
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let collection: Collection<Document> = database.collection(format!("Round {}", round).as_str());
    println!("{}", match_id);
    let filter = doc! {
        "match_id": match_id,
        "tag": {
           "$ne": other_tag
        }
    };
    println!("Filter: {:?}", filter);
    match collection.find_one(filter, None).await {
        Ok(Some(enemy)) => {
            println!("Found enemy!");
            Some(enemy)
        }
        Ok(None) => {
            println!("Can't find enemy!");
            None
        }
        Err(err) => {
            eprintln!("Error while querying database: {:?}", err);
            None
        }
    }
}

pub fn is_mannequin(enemy: &Document) -> bool {
    enemy.get("tag").unwrap() == &Bson::Null
}
