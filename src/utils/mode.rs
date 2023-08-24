use std::collections::HashMap;

pub fn get_mode_icon() -> HashMap<String,String>{
  let mut mode:HashMap<String,String> = HashMap::new();
  mode.insert(String::from("brawlBall"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_brawl_ball.png"));
  mode.insert(String::from("bounty"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_bounty.png"));
  mode.insert(String::from("gemGrab"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_gem_grab.png"));
  mode.insert(String::from("wipeOut"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_wipe_out.png"));
  mode.insert(String::from("heist"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_heist.png"));
  mode.insert(String::from("hotZone"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_hot_zone.png"));
  mode.insert(String::from("knockout"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_knockout.png"));
  mode.insert(String::from("siege"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_siege.png"));
  mode.insert(String::from("bounty"), String::from("https://cdn.brawlstats.com/event-icons/event_mode_bounty.png"));
  mode.insert(String::from("soloShowdown"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_solo_showdown.png"));
  mode.insert(String::from("duoShowdown"),String::from("https://cdn.brawlstats.com/event-icons/event_mode_duo_showdown.png"));
  
  mode
}
