use super::mannequin::update_mannequin;
use super::region::Region;
use crate::Error;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

pub async fn assign_match_id(_region: &Region, database: &Database) -> Result<(), Error> {
    let collection: Collection<Document> = database.collection("Round 1");
    let mut byes_counter = collection
        .count_documents(doc! {"name": "Mannequin"}, None)
        .await?;
    print!("{}", byes_counter);
    let mut player_cursor = collection
        .find(
            doc! {
                    "name": { "$ne": "Mannequin" },
                    "match_id": null
            },
            None,
        )
        .await?;
    let mut double_match_id: i32 = 2;
    //So here is the math behind this:
    //We want 2 consecutive players to be assigned with the same match_id
    //So we assign double_match_id (declared as integer i32) and start with 2, for every iteration, this increments by 1
    //And we take half of the value to get the actual match_id
    //Therefore, 1st player: 2/2=1; 2nd player: 3/2=1; 3rd player 4/2=2; 4th player: 5/2=2; and so on
    while let Ok(Some(mut player)) = player_cursor.try_next().await {
        let match_id = double_match_id / 2;
        let update = doc! {
            "$set": {"match_id": match_id}
        };
        player.insert("match_id", match_id);
        collection
            .update_one(doc! { "_id": player.get_object_id("_id")? }, update, None)
            .await?;

        //not a while loop here because we need to assign match_id to mannequin after assign an id to player
        if byes_counter > 0 {
            collection
                .update_one(
                    doc! {"match_id": null, "name": "Mannequin"},
                    update_mannequin(match_id),
                    None,
                )
                .await?;
            byes_counter -= 1;
            double_match_id += 1;
        }
        double_match_id += 1;
    }
    Ok(())
}

pub fn update_match_id(mut player: Document) -> Document {
    let old_match_id = player.get_i32("match_id").unwrap();
    let new_match_id = (old_match_id + 1) / 2;
    player.insert("match_id", new_match_id);
    println!("Match id is updated!");
    player
}
