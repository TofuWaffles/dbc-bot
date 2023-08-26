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
/// use serde_json::json;
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
        "brawlBall" => "https://cdn.brawlstats.com/event-icons/event_mode_brawl_ball.png",
        "bounty" => "https://cdn.brawlstats.com/event-icons/event_mode_bounty.png",
        "gemGrab" => "https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png",
        "wipeout" => "https://cdn.brawlstats.com/event-icons/event_mode_wipe_out.png",
        "heist" => "https://cdn.brawlstats.com/event-icons/event_mode_heist.png",
        "hotZone" => "https://cdn.brawlstats.com/event-icons/event_mode_hot_zone.png",
        "knockout" => "https://cdn.brawlstats.com/event-icons/event_mode_knockout.png",
        "siege" => "https://cdn.brawlstats.com/event-icons/event_mode_siege.png",
        "soloShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_solo_showdown.png",
        "duoShowdown" => "https://cdn.brawlstats.com/event-icons/event_mode_duo_showdown.png",
        _ => {
            "https://cdn.discordapp.com/emojis/1133867752155779173.webp?size=4096&quality=lossless"
        }
    }
}
