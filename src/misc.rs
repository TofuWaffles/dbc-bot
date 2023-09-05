use poise::serenity_prelude::Colour;
use std::error::Error;
use std::fmt;

/// A trait for stripping quotes from a string.
pub trait QuoteStripper {
    /// Strip double quotes from the string and return a new String.
    fn strip_quote(&self) -> String;
}

impl QuoteStripper for String {
    /// Strip double quotes from the string and return a new String.
    ///
    /// # Examples
    ///
    /// ```
    /// let s = String::from("\"Hello, world!\"");
    /// let stripped = s.strip_quote();
    /// assert_eq!(stripped, "Hello, world!");
    /// ```
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

/// This function converts a difficulty level represented as a serde_json::Value into its corresponding
/// textual representation.
///
/// # Arguments
///
/// * `num` - A reference to a serde_json::Value representing the difficulty level.
///
/// # Returns
///
/// A String representing the textual description of the difficulty level. If the provided numeric value
/// does not correspond to a recognized difficulty level, a default message is returned.
///
/// # Examples
///
/// ```
/// use poise::serenity_prelude::json::json;
///
/// let num = json!(3);
/// let difficulty = get_difficulty(&num);
/// assert_eq!(difficulty, "Expert");
/// ```
pub fn get_difficulty(num: &serde_json::Value) -> String {
    let option: i32 = serde_json::from_value(num.clone()).unwrap();
    match option {
        0 => "Easy".to_string(),
        1 => "Normal".to_string(),
        2 => "Hard".to_string(),
        3 => "Expert".to_string(),
        4 => "Master".to_string(),
        5 => "Insane".to_string(),
        6 => "Insane II".to_string(),
        7 => "Insane III".to_string(),
        8 => "Insane IV".to_string(),
        9 => "Insane V".to_string(),
        10 => "Insane VI".to_string(),
        11 => "Insane VII".to_string(),
        12 => "Insane VIII".to_string(),
        13 => "Insane IX".to_string(),
        14 => "Insane X".to_string(),
        15 => "Insane XI".to_string(),
        16 => "Insane XII".to_string(),
        17 => "Insane XIII".to_string(),
        18 => "Insane XIV".to_string(),
        19 => "Insane XV".to_string(),
        20 => "Insane XVI".to_string(),
        _ => "Congratulations, either we were wrong, or you unlocked new difficulty".to_string(),
    }
}

/// This function returns the URL of a game mode icon based on the provided event name.
///
/// # Arguments
///
/// * `event_name` - A string slice containing the name of the event.
///
/// # Returns
///
/// An `Option<&str>` representing the URL of the event icon. If the event name is recognized,
/// it returns `Some(&str)` with the URL; otherwise, it returns `None`.
///
/// # Examples
///
/// ```
/// let event_name = "gemGrab";
/// match get_mode_icon(event_name) {
///     Some(url) => println!("URL for {} is {}", event_name, url),
///     None => println!("Event name {} not found.", event_name),
/// }
/// ```
pub fn get_mode_icon(event_name: &serde_json::Value) -> &str {
    let binding = event_name.to_string().strip_quote();
    let event_link: &str = binding.as_str();

    // Match the event_name to known event names and return the corresponding URL as Some(&str)
    match event_link {
        "brawlBall" => "https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png",
        "bounty" => "https://cdn.brawlstats.com/event-icons/event_mode_bounty.png",
        "gemGrab" => "https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png",
        "wipeout" => "https://cdn.brawlstats.com/event-icons/event_mode_wipeout.png",
        "heist" => "https://cdn.brawlstats.com/event-icons/event_mode_heist.png",
        "hotZone" => "https://cdn.brawlstats.com/event-icons/event_mode_hot_zone.png",
        "knockout" => "https://cdn.brawlstats.com/event-icons/event_mode_knockout.png",
        "siege" => "https://cdn.brawlstats.com/event-icons/event_mode_siege.png",
        "raid" => "https://cdn.brawlstats.com/event-icons/event_mode_raid.png",
        "soloShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_showdown.png",
        "duoShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_showdown.png",
        _ => {
            "https://cdn.discordapp.com/emojis/1133867752155779173.webp?size=4096&quality=lossless"
        }
    }
}

/// Converts a string result into a corresponding color represented as a `poise::serenity_prelude::Colour` struct.
///
/// This function takes a `result` parameter, which is a string indicating the result of an event.
/// It matches the input string to predefined cases and returns a `poise::serenity_prelude::Colour` value representing the
/// associated color.
///
/// # Arguments
///
/// * `result` - A string representing the result of an event ("victory", "defeat", "draw").
///
/// # Returns
///
/// A `poise::serenity_prelude::Colour` struct representing the color associated with the input result. If the input result
/// is not recognized (i.e., not "victory", "defeat", or "draw"), the function returns a default
/// color (black).
///
/// # Examples
///
/// ```
/// use your_crate_name::color;
/// use poise::serenity_prelude::Colour;
///
/// let victory_color = color("victory".to_string());
/// assert_eq!(victory_color, Colour::new(0x00800)); // Green
///
/// let defeat_color = color("defeat".to_string());
/// assert_eq!(defeat_color, Colour::new(0xFF0000)); // Red
///
/// let draw_color = color("draw".to_string());
/// assert_eq!(draw_color, Colour::new(0xFFFFFF)); // White
///
/// let unknown_color = color("unknown".to_string());
/// assert_eq!(unknown_color, Colour::new(0x000000)); // Default (black)
/// ```
pub fn get_color(result: String) -> Colour {
    match result.as_str() {
        "victory" => Colour::new(u32::from_str_radix("90EE90", 16).unwrap()), // Green
        "defeat" => Colour::new(u32::from_str_radix("FF0000", 16).unwrap()),  // Red
        "draw" => Colour::new(u32::from_str_radix("FFFFFF", 16).unwrap()),    // White
        _ => Colour::new(000000), // Default color (black) for unknown cases
    }
}

// Define a custom error type for your application
#[derive(Debug)]
pub struct CustomError(pub String);
/// Implements the `fmt::Display` trait for the `CustomError` struct.
///
/// This implementation allows instances of the `CustomError` struct to be formatted as strings
/// when using the `format!`, `println!`, or `write!` macros. It displays the error message
/// contained within the `CustomError` struct.
///
/// # Example
///
/// ```
/// use your_crate_name::CustomError;
///
/// let error = CustomError("An error occurred".to_string());
/// println!("Error: {}", error); // Prints "Error: CustomError: An error occurred"
/// ```
impl fmt::Display for CustomError {
    /// Formats the `CustomError` instance as a string.
    ///
    /// # Arguments
    ///
    /// * `self` - The `CustomError` instance to format.
    /// * `f` - The formatter used to write the formatted output.
    ///
    /// # Returns
    ///
    /// A `fmt::Result` indicating whether the formatting operation was successful.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomError: {}", self.0)
    }
}

impl Error for CustomError {}
