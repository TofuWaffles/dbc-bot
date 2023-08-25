use std::collections::HashMap;

/// Returns a HashMap containing mode names as keys and their corresponding icon URLs as values.
///
/// This function initializes a HashMap with mode names as keys (e.g., "brawlBall", "bounty") and
/// their corresponding icon URLs as values. It is typically used to map game modes to their
/// respective icons for display purposes.
///
/// # Returns
///
/// A HashMap where the keys are mode names (as strings) and the values are URLs (as strings)
/// pointing to mode icons.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use your_crate_name::get_mode_icon;
///
/// let mode_icons = get_mode_icon();
///
/// // Accessing the URL for the "brawlBall" mode icon
/// if let Some(url) = mode_icons.get("brawlBall") {
///     println!("URL for Brawl Ball icon: {}", url);
/// } else {
///     println!("Brawl Ball icon not found.");
/// }
/// ```
pub fn get_mode_icon() -> HashMap<String,String>{
  println!("This is called");
  let mut mode:HashMap<String,String> = HashMap::new();
  mode.insert(String::from("brawlBall"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_brawl_ball.png"));
  mode.insert(String::from("bounty"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_bounty.png"));
  mode.insert(String::from("gemGrab"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png"));
  mode.insert(String::from("wipeout"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_wipe_out.png"));
  mode.insert(String::from("heist"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_heist.png"));
  mode.insert(String::from("hotZone"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_hot_zone.png"));
  mode.insert(String::from("knockout"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_knockout.png"));
  mode.insert(String::from("siege"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_siege.png"));
  mode.insert(String::from("bounty"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_bounty.png"));
  mode.insert(String::from("soloShowdown"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_solo_showdown.png"));
  mode.insert(String::from("duoShowdown"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_duo_showdown.png"));
  
  mode
}

 