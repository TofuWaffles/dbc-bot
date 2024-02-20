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

pub fn get_mode_icon(event_name: String) -> String {
    // Match the event_name to known event names and return the corresponding URL as Some(&str)
    let event = match event_name.as_str() {
        "brawlBall" => "https://cdn-old.brawlify.com/gamemode/Brawl-Ball.png",
        "bounty" => "https://cdn-old.brawlify.com/gamemode/Bounty.png",
        "gemGrab" => "https://cdn-old.brawlify.com/gamemode/Gem-Grab.png",
        "wipeout" => "https://cdn-old.brawlify.com/gamemode/Wipeout.png",
        "heist" => "https://cdn-old.brawlify.com/gamemode/Heist.png",
        "hotZone" => "https://cdn-old.brawlify.com/gamemode/Hot-Zone.png",
        "knockout" => "https://cdn-old.brawlify.com/gamemode/Knockout.png",
        "siege" => "https://cdn-old.brawlify.com/gamemode/Siege.png",
        "raid" => "https://cdn.brawlstats.com/event-icons/event_mode_raid.png",
        "soloShowdown" => "https://cdn-old.brawlify.com/gamemode/Solo-Showdown.png",
        "duoShowdown" => "https://cdn-old.brawlify.com/gamemode/Duo-Showdown.png",
        _ => {
            "https://cdn.discordapp.com/emojis/1133867752155779173.webp?size=4096&quality=lossless"
        }
    };
    event.to_string()
}

pub fn get_player_icon(icon_id: i64) -> String {
    format!("https://cdn.brawlify.com/profile/{icon_id}.png?v=1")
}
