use chrono::{Local, Timelike};
use mongodb::bson::Bson;
use poise::serenity_prelude::{Colour, Timestamp};
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use strum_macros::EnumIter;
/// A trait for stripping quotes from a string.
pub trait QuoteStripper {
    /// Strip double quotes from the string and return a new String.
    fn strip_quote(&self) -> String;
}

impl QuoteStripper for String {
    fn strip_quote(&self) -> String {
        let mut result = String::new();

        for c in self.chars() {
            if c != '"' {
                result.push(c);
            }
        }

        result
    }
}

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
            Bson::String(s) => Some(Self::from_str(s).unwrap()),
            _ => None,
        }
    }
    /// Returns the short name of the region, e.g. `NASA`, `EU`, `APAC`.
    pub fn short(&self) -> String {
        format!("{:?}", self)
    }
    /// Returns the full name of the region, e.g. `North America & South America`, `Europe`, `Asia & Oceania`.
    pub fn full(&self) -> String {
        format!("{}", self)
    }

    pub fn get_emoji(&self) -> String {
        match self {
            Region::NASA => "🌎",
            Region::EU => "🌍",
            Region::APAC => "🌏",
        }
        .to_string()
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

pub fn get_color(result: String) -> Colour {
    match result.as_str() {
        "victory" => Colour::new(0x90EE90), // Green
        "defeat" => Colour::new(0xFF0000),  // Red
        "draw" => Colour::new(0xFFFFFF),    // White
        _ => Colour::new(0x000000),         // Default color (black) for unknown cases
    }
}

// Define a custom error type for your application
#[derive(Debug)]
pub struct CustomError(pub String);

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomError: {}", self.0)
    }
}

impl Error for CustomError {}

trait BsonExtensions {
    fn as_u32(&self) -> Option<u32>;
}

impl BsonExtensions for Bson {
    fn as_u32(&self) -> Option<u32> {
        match self {
            Bson::Int32(value) => {
                if *value >= 0 {
                    Some(*value as u32)
                } else {
                    None
                }
            }
            Bson::Int64(value) => {
                if *value >= 0 && *value <= u32::MAX as i64 {
                    Some(*value as u32)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct Time {
    pub years: u32,
    pub months: u8,
    pub days: u8,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
    pub time: Option<String>,
}

impl Time {
    pub fn standardising(time: &str) -> Time {
        #[allow(non_snake_case)]
        let T = time.find('T').unwrap();
        let mut time = Time {
            years: time[0..T - 4].parse::<u32>().unwrap(),
            months: time[T - 4..T - 2].parse::<u8>().unwrap(),
            days: time[T - 2..T].parse::<u8>().unwrap(),
            hours: time[T + 1..T + 3].parse::<u8>().unwrap(),
            minutes: time[T + 3..T + 5].parse::<u8>().unwrap(),
            seconds: time[T + 5..T + 7].parse::<u8>().unwrap(),
            milliseconds: time[T + 8..T + 11].parse::<u16>().unwrap(),
            time: None,
        };
        time.format();
        time
    }

    pub fn format(&mut self) {
        self.time = Some(format!(
            "{}-{}-{}T{}:{}:{}Z",
            self.years, self.months, self.days, self.hours, self.minutes, self.seconds
        ))
    }

    pub fn get_unix(&self) -> Timestamp {
        let time: &str = &format!(
            "{}-{}-{}T{}:{}:{}Z",
            self.years, self.months, self.days, self.hours, self.minutes, self.seconds
        );
        Timestamp::parse(time).unwrap()
    }

    pub fn get_relative(&self) -> String {
        let now = Local::now();
        let hours = now.hour() - (self.hours as u32);
        let minutes = now.minute() - (self.minutes as u32);
        let seconds = now.second() - (self.seconds as u32);
        format!("{}h {}m {}s ago", hours, minutes, seconds)
    }
}
