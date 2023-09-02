use super::mannequin::add_mannequin;
use crate::misc::Region;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

pub async fn assign_match_id(
    region: &Region,
    database: &Database,
    byes: i32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let collection: Collection<Document> = database.collection("Player");
    let mut player_cursor = collection.find(doc! {"match_id": null}, None).await?;
    let mut double_match_id: i32 = 2;
    //So here is the math behind this:
    //We want 2 consecutive players to be assigned with the same match_id
    //So we assign double_match_id (declared as integer i32) and start with 2, for every iteration, this increments by 1
    //And we take half of the value to get the actual match_id
    //Therefore, 1st player: 2/2=1; 2nd player: 3/2=1; 3rd player 4/2=2; 4th player: 5/2=2; and so on
    let mut byes_counter = byes;

    while let Ok(Some(mut document)) = player_cursor.try_next().await {
        let match_id = double_match_id / 2;
        let update = doc! {
            "$set": {"match_id": match_id}
        };
        document.insert("match_id", match_id);
        collection
            .update_one(
                doc! { "_id": document.get_object_id("_id")? },
                update.clone(),
                None,
            )
            .await?;

        //not a while loop here because we need to assign match_id to mannequin after assign an id to player
        if byes_counter > 0 {
            add_mannequin(region, Some(match_id), None);
            byes_counter -= 1;
            double_match_id += 1;
        }
        double_match_id += 1;
    }
    Ok(())
}

pub async fn update_match_id() {}
