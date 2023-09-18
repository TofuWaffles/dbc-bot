use crate::misc::get_icon;
use std::error::Error;

use super::helper::align::{center_x, center_y};
use super::helper::draw::{draw_rec, make_text_image};
use super::helper::fetch::fetch_image;
use image::imageops::FilterType::Nearest;
use image::{imageops, DynamicImage};
use mongodb::bson::Document;

async fn create_battle_image(
    player1: &Document,
    player2: &Document,
    round: i32,
    match_id: i32,
    mode: &str,
) -> DynamicImage {
    // Open the background image
    let base = image::open("src\\visual\\asset\\battle_background.jpg")
        .expect("Failed to open background image");
    let vs = image::open("src\\visual\\asset\\versus.png")
        .expect("Failed to open vs image")
        .resize(150, 150, Nearest);
    // Fetch player icons and mode icon asynchronously
    let icon1_url = get_icon("player")(player1.get("icon").unwrap().to_string());
    let icon1 = fetch_image(icon1_url, (Some(200), Some(200))).await;
    let icon2_url = get_icon("player")(player2.get("icon").unwrap().to_string());
    let icon2 = fetch_image(icon2_url, (Some(200), Some(200))).await;
    let name1_color = u32::from_str_radix(
        &(player1.get("name_color").unwrap().as_str().unwrap()[2..]),
        16,
    )
    .unwrap();
    let name2_color = u32::from_str_radix(
        &(player2.get("name_color").unwrap().as_str().unwrap()[2..]),
        16,
    )
    .unwrap();
    let name1 = make_text_image(
        player1.get("name").unwrap().as_str().unwrap(),
        30,
        &name1_color,
    );
    let name2 = make_text_image(
        player2.get("name").unwrap().as_str().unwrap(),
        30,
        &name2_color,
    );
    let tag1 = make_text_image(
        player1.get("tag").unwrap().as_str().unwrap(),
        30,
        &name1_color,
    );
    let tag2 = make_text_image(
        player2.get("tag").unwrap().as_str().unwrap(),
        30,
        &name2_color,
    );
    let mode_url = get_icon("mode")(mode.to_string());

    let mode_icon = fetch_image(mode_url, (None, None)).await;

    // Create text images for title, mode, VS, and footer
    let title = make_text_image(
        &format!("Round {} - Match {}", round, match_id),
        50,
        &0xFFFF99,
    );
    let mode_text = make_text_image(&mode.to_uppercase(), 40, &0xFFFFFF);
    let footer = make_text_image("Best of 2", 50, &0xFFFFFF);

    // Clone the base image to create an overlay
    let mut overlay = base.clone();

    // Calculate positions for various elements

    // Title Position
    let title_x = center_x(base.width() as i64, title.width() as i64);
    let title_y = 100;

    // Mode Icon Position
    let mode_icon_x = 0;
    let mode_icon_y = 0;

    // Mode Text Position
    let mode_text_x = mode_icon.width() as i64 + 10;
    let mode_text_y = center_y(mode_icon.height() as i64, mode_text.height() as i64);

    // Player Icons Positions
    let icon1_x = 50;
    let icon1_y = center_y(base.height() as i64, icon1.height() as i64);
    let icon2_x = base.width() as i64 - icon1_x - icon2.width() as i64;
    let icon2_y = center_y(base.height() as i64, icon2.height() as i64);

    // Player Names Positions
    let name1_x = center_x(icon1_x * 2 + icon1.width() as i64, name1.width() as i64);
    let name1_y = icon1_y + icon1.height() as i64 + 10;
    let name2_x = center_x(icon2_x * 2 + icon2.width() as i64, name2.width() as i64);
    let name2_y = icon2_y + icon2.height() as i64 + 10;

    // Player Tags Positions
    let tag1_x = center_x(name1_x * 2 + name1.width() as i64, tag1.width() as i64);
    let tag1_y = name1_y + name1.height() as i64 + 10;
    let tag2_x = center_x(name2_x * 2 + name2.width() as i64, tag2.width() as i64);
    let tag2_y = name2_y + name2.height() as i64 + 10;

    // VS Text Position
    let vs_x = center_x(base.width() as i64, vs.width() as i64);
    let vs_y = center_y(base.height() as i64, vs.height() as i64);

    // Footer Position
    let footer_x = center_x(base.width() as i64, footer.width() as i64);
    let footer_y = base.height() as i64 - footer.height() as i64 - 50;

    // Overlay elements onto the base image
    imageops::overlay(&mut overlay, &draw_rec(300, 100, 0xFE1BB8), 0, 0);
    imageops::overlay(&mut overlay, &title, title_x, title_y);
    imageops::overlay(&mut overlay, &mode_icon, mode_icon_x, mode_icon_y);
    imageops::overlay(&mut overlay, &mode_text, mode_text_x, mode_text_y);
    imageops::overlay(&mut overlay, &icon1, icon1_x, icon1_y);
    imageops::overlay(&mut overlay, &icon2, icon2_x, icon2_y);
    imageops::overlay(&mut overlay, &name1, name1_x, name1_y);
    imageops::overlay(&mut overlay, &name2, name2_x, name2_y);
    imageops::overlay(&mut overlay, &tag1, tag1_x, tag1_y);
    imageops::overlay(&mut overlay, &tag2, tag2_x, tag2_y);
    imageops::overlay(&mut overlay, &vs, vs_x, vs_y);
    imageops::overlay(&mut overlay, &footer, footer_x, footer_y);
    overlay
}

pub async fn generate_pre_battle_img(
    player1: &Document,
    player2: &Document,
    config: &Document,
) -> Result<DynamicImage, Box<dyn Error>> {
    let mode = config.get("mode").unwrap().as_str().unwrap();
    let round = config.get("round").unwrap().as_i32().unwrap();
    let match_id = player1.get("match_id").unwrap().as_i32().unwrap();
    let result = create_battle_image(player1, player2, round, match_id, mode).await;
    Ok(result)
}
