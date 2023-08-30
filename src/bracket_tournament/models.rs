use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, Default, poise::ChoiceParameter, Serialize, Deserialize)]
pub enum Region {
    #[default]
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

impl Region {
    pub fn get_enum_as_string(&self) -> String {
        match self {
            Region::NASA => "NASA".to_string(),
            Region::EU => "EU".to_string(),
            Region::APAC => "APAC".to_string(),
        }
    }

    pub fn from_str(region: &str) -> Option<Region> {
        match region {
            "NASA" => Some(Region::NASA),
            "EU" => Some(Region::EU),
            "APAC" => Some(Region::APAC),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub tag: String,
    pub discord_id: String,
    pub region: Region,
}

#[derive(Debug, Clone, Default)]
pub struct BracketPair {
    pub id: String,
    pub player1_id: Option<String>, // These are the Discord IDs of the players
    pub player2_id: Option<String>,
    pub upper_bracket: Option<ParentBracketPairRef>,
    pub winner_id: Option<String>, // Likewise, this stores the winner's Discord ID
    pub region: Region,
}

pub type ParentBracketPairRef = Rc<RefCell<BracketPair>>;

impl BracketPair {
    pub fn new(
        id: String,
        player1_id: Option<String>,
        player2_id: Option<String>,
        upper_bracket: Option<ParentBracketPairRef>,
        region: Region,
    ) -> Self {
        BracketPair {
            id,
            player1_id,
            player2_id,
            upper_bracket,
            winner_id: None,
            region,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TournamentStatus {
    pub ongoing: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub player_count: u32,
    pub region: Region,
}
