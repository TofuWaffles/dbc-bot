use mongodb::bson::{Bson, Document};
use tracing::error;

pub fn get_round(config: &Document) -> Option<String> {
    let round = match config.get("round") {
        Some(round) => {
            if let Bson::Int32(0) = round {
                Some("Players".to_string())
            } else {
                Some(format!("Round {}", round.as_i32().unwrap()))
            }
        }
        None => {
            error!("Error while getting round from config");
            None
        }
    };

    round
}
