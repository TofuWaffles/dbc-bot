use std::str::FromStr;
use mongodb::bson::Bson;
use strum_macros::EnumIter;

// Define an enum called `Region` to represent geographical regions.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, poise::ChoiceParameter, EnumIter, Eq, Hash, PartialEq, Clone)]
pub enum Region {
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

impl Region {
    pub fn find_key(name: &str) -> Option<Region> {
        match name {
            "NASA" => Some(Region::NASA),
            "EU" => Some(Region::EU),
            "APAC" => Some(Region::APAC),
            _ => None,
        }
    }
    pub fn from_bson(bson: &Bson) -> Option<Self> {
        match bson {
            Bson::String(s) => {
                Some(Self::from_str(s).unwrap())
            }
            _ => None,
        }
    }
}

#[derive(Debug, poise::ChoiceParameter, EnumIter, Eq, Hash, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Mode {
    #[name = "Wipeout"]
    wipeout,
    #[name = "Brawl Ball"]
    brawlBall,
    #[name = "Gem Grab"]
    gemGrab,
    #[name = "Heist"]
    heist,
    #[name = "Hot Zone"]
    hotZone,
    #[name = "Knockout"]
    knockout,
    #[name = "Siege"]
    siege,
    #[name = "Bounty"]
    bounty,
}

impl Mode {
    pub fn find_key(name: &str) -> Option<Mode> {
        match name {
            "Wipeout" | "wipeout" => Some(Mode::wipeout),
            "Brawl Ball" | "brawlBall" => Some(Mode::brawlBall),
            "Gem Grab" | "gemGrab" => Some(Mode::gemGrab),
            "Heist" | "heist" => Some(Mode::heist),
            "Hot Zone" | "hotZone" => Some(Mode::hotZone),
            "Knockout" | "knockout" => Some(Mode::knockout),
            "Siege" | "siege" => Some(Mode::siege),
            "Bounty" | "bounty" => Some(Mode::bounty),
            _ => None,
        }
    }
}
