use super::model::{self, *};
use crate::brawlstars::getters::get_mode_icon;
use crate::brawlstars::getters::get_player_icon;
use crate::Error;
use dbc_bot::CustomError;
use image::{imageops, DynamicImage};
use mongodb::bson::Document;
use std::env;
use tracing::{error, info};
const FONT_SIZE: u8 = 30;
const ICON_SIZE: i64 = 200;
async fn create_battle_image(
    player1: &Document,
    player2: &Document,
    round: i32,
    match_id: i32,
    mode: &str,
) -> Result<DynamicImage, Error> {
    let current_dir = match env::current_dir() {
        Ok(dir) => {
            info!("Current directory: {:?}", dir);
            dir
        }
        Err(e) => {
            error!("Failed to get current directory: {}", e);
            return Err(Box::new(CustomError(format!("{e}"))));
        }
    };
    
    let bg_path = match current_dir
        .join("assets/battle_background.jpg")
        .into_os_string()
        .into_string()
    {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get background img path: {:?}", e);
            return Err(Box::new(CustomError(format!("{:?}", e))));
        }
    };
    let mut img = model::BSImage::new(None, None, bg_path, Some("Prebattle"));

    // Open the versus img
    let mut vs = model::Component::new(
        image::open(current_dir.join("assets/versus.png"))?.resize(
            150,
            150,
            imageops::FilterType::Nearest,
        ),
        None,
        None,
        Some("versus"),
    );
    vs.set_center_x(img.width);
    vs.set_center_y(img.height);

    // Fetch player icons and mode icon asynchronously
    let mut icon1 = model::Component::new(
        model::CustomImage::new(
            get_player_icon(player1.get_i64("icon").unwrap()),
            Some(ICON_SIZE),
            Some(ICON_SIZE),
        )
        .build()
        .await?,
        Some(50),
        None,
        Some("icon1"),
    );
    icon1.set_center_y(img.height);

    let mut icon2 = model::Component::new(
        model::CustomImage::new(
            get_player_icon(player2.get_i64("icon").unwrap()),
            Some(ICON_SIZE),
            Some(ICON_SIZE),
        )
        .build()
        .await?,
        None,
        None,
        Some("icon2"),
    );
    icon2.set_x(img.width - icon1.x - icon2.width());
    icon2.set_center_y(img.height);

    let mut name1 = model::Component::new(
        model::Text {
            text: player1.get_str("name").unwrap().to_string(),
            font_size: FONT_SIZE,
            font_color: u32::from_str_radix(&(player1.get_str("name_color").unwrap()[2..]), 16)?,
        }
        .build()
        .await?,
        None,
        None,
        Some("name1"),
    );
    name1.set_relative_center_x(&icon1);
    name1.set_y(icon1.y + icon1.height() + 10);

    let mut name2 = model::Component::new(
        model::Text::new(
            player2.get_str("name").unwrap(),
            FONT_SIZE,
            u32::from_str_radix(&(player2.get_str("name_color").unwrap()[2..]), 16)?,
        )
        .build()
        .await?,
        None,
        None,
        Some("name2"),
    );
    name2.set_relative_center_x(&icon2);
    name2.set_y(icon2.y + icon2.height() + 10);

    let mut tag1 = model::Component::new(
        model::Text {
            text: player1.get_str("tag").unwrap().to_string(),
            font_size: FONT_SIZE,
            font_color: u32::from_str_radix(&(player1.get_str("name_color").unwrap()[2..]), 16)?,
        }
        .build()
        .await?,
        None,
        None,
        Some("tag1"),
    );
    tag1.set_relative_center_x(&name1);
    tag1.set_y(name1.y + name1.height() + 10);

    let mut tag2 = model::Component::new(
        model::Text::new(
            player2.get_str("tag").unwrap(),
            FONT_SIZE,
            u32::from_str_radix(&(player2.get_str("name_color").unwrap()[2..]), 16)?,
        )
        .build()
        .await?,
        None,
        None,
        Some("tag2"),
    );
    tag2.set_relative_center_x(&name2);
    tag2.set_y(name2.y + name2.height() + 10);

    // Create text images for title, mode, VS, and footer
    let mut title = model::Component::new(
        model::Text::new(format!("Round {round} - Match {match_id}"), 50, 0xFFFFFF)
            .build()
            .await?,
        None,
        Some(100),
        Some("title"),
    );
    title.set_center_x(img.width);
    // Create mode components
    let mut mode_bg = model::Component::new(
        model::Parallelogram{
            top: 1000,
            bottom: 800,
            height: 200,
            color: 0xFE1AB6FF
        }.build().await?,
        None,
        Some(0),
        Some("mode_bg")
    );
    mode_bg.set_center_x(img.width);
    let mut mode_icon = model::Component::new(
        model::CustomImage::new(get_mode_icon(mode.to_string()), Some(50), Some(50))
            .build()
            .await?,
        Some(mode_bg.x + 10),
        Some(mode_bg.y),
        Some("mode_icon"),
    );
    imageops::overlay(&mut mode_bg.img, &mut mode_icon.img, 0, 0);

    let mut mode_overlay = model::Component::new(
        model::Parallelogram{
            top: 1000,
            bottom: 800,
            height: 200,
            color: 0xFE1AB680
        }.build().await?,
        Some(0),
        Some(0),
        Some("mode_comp")
    );
    imageops::overlay(&mut mode_bg.img, &mut mode_overlay.img, 0, 0);
    
    let mut mode_text = model::Component::new(
        model::Text::new(mode.to_uppercase(), 40, 0xFFFFFF)
            .build()
            .await?,
        Some(mode_icon.width() as i64 + 10),
        Some(0),
        Some("mode_text"),
    );
    let text_x = mode_text.get_center_x(mode_bg.width());
    imageops::overlay(&mut mode_bg.img, & mode_text.img, text_x , 0);
   


    let mut footer = model::Component::new(
        model::Text::new("Best of 2", 30, 0xFFFFFF).build().await?,
        None,
        None,
        Some("footer"),
    );
    footer.set_center_x(img.width);
    footer.set_y(img.height - footer.height() - 50);

    // Component elements onto the base img
    img.add_overlay(title);
    img.add_overlay(mode_bg);
    img.add_overlay(icon1);
    img.add_overlay(icon2);
    img.add_overlay(name1);
    img.add_overlay(name2);
    img.add_overlay(tag1);
    img.add_overlay(tag2);
    img.add_overlay(vs);
    img.add_overlay(footer);

    // Build the final composed img
    Ok(img.build())
}

pub async fn generate_pre_battle_img(
    player1: &Document,
    player2: &Document,
    config: &Document,
) -> Result<DynamicImage, Error> {
    let mode = config.get_str("mode").unwrap();
    let round = config.get_i32("round").unwrap();
    let match_id = player1.get_i32("match_id").unwrap();
    create_battle_image(player1, player2, round, match_id, mode).await
}
