use mongodb::{
    bson::{doc, Bson::Null, Document},
    Collection, Database,
};

use crate::bracket_tournament::region::Mode;


pub fn make_config() -> Document {
    let config = doc! {
      "registration": true,
      "round": 1,
      "mode": Null,
      "map": Null
    };
    config
}

pub fn set_config(mode: &Mode, map: &String) -> Document {
    let config = doc! {
      "$set": {
        "mode": format!("{:?}", mode),
        "map": map
      }
    };
    config
}

pub async fn get_config(db: &Database) -> Document {
    let collection: Collection<Document> = db.collection("Config");
    let config = collection.find_one(None, None).await.unwrap().unwrap();
    config
}

pub fn disable_registration() -> Document {
    let config = doc! {
      "$set": {
        "registration": false
      }
    };
    config
}

pub fn enable_registration() -> Document {
    let config = doc! {
      "$set": {
        "registration": true
      }
    };
    config
}

pub fn update_round(round: Option<i32>) -> Document {
    match round {
        Some(round) => {
            let config = doc! {
              "$set": {
                "round": round
              }
            };
            config
        }
        None => {
            let config = doc! {
              "$inc": {
                "round": 1
              }
            };
            config
        }
    }
}
