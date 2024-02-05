use crate::misc::get_icon;
use std::error::Error;

use super::helper::align::{center_x, center_y};
use super::helper::draw::{draw_rec, make_text_image};
use super::helper::fetch::fetch_image;
use image::imageops::FilterType::Nearest;
use image::{imageops, DynamicImage};
use mongodb::bson::Document;

fn preset(player1: &Document, player2: &Document) -> DynamicImage {
    let base = image::open("src\\visual\\asset\\battle_log.png").expect("Failed to open background image");
    let icon1_url = get_icon("player")(player1.get("icon").unwrap().to_string());
    let icon1 = fetch_image(icon1_url, (Some(200), Some(200))).await;
    let icon2_url = get_icon("player")(player2.get("icon").unwrap().to_string());
    let icon2 = fetch_image(icon2_url, (Some(200), Some(200))).await;
}