use mongodb::bson::{Bson, Document};

pub fn get_round(config: &Document) -> String {
    let round = match config.get("round") {
        Some(round) => {
            if let Bson::Int32(0) = round {
                "Players".to_string()
            } else {
                format!("Round {}", round.as_i32().unwrap())
            }
        }
        _ => unreachable!("Round not found in config!"),
    };

    round
}
