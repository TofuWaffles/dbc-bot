use mongodb::{Database, bson::{Document, doc}, Collection};



pub async fn update_battle(database: &Database, round: i32, match_id: i32) -> Result<(),Box<dyn std::error::Error + Send + Sync>>{
    let current_round: Collection<Document> = database.collection(format!("Round {}", round).as_str());
    let filter = doc! {
        "match_id": match_id
    };
    let update = doc! {
        "$set": {
           "battle": true
        }
    };
    current_round.update_many(filter, update, None).await?;
    println!("Battle is updated!");
    
    Ok(())
}