use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};

pub fn make_config() -> Document {
    let config = doc! {
      "registration": true,
      "round": 1
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
