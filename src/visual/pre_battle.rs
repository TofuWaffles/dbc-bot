use super::model::{self, *};
use crate::brawlstars::getters::get_mode_icon;
use crate::brawlstars::getters::get_player_icon;
use crate::Error;
use dbc_bot::CustomError;
use image::{imageops, DynamicImage};
use mongodb::bson::Document;
use std::env;
use std::io::Cursor;
use tracing::error;
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
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to get current directory: {}", e);
            return Err(Box::new(CustomError(format!("{e}"))));
        }
    };

    let bg_path = match current_dir
        .join("assets/battle/battle_background.png")
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
    let vs_path = match current_dir
        .join("assets/battle/versus.png")
        .into_os_string()
        .into_string()
    {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to get versus img path: {:?}", e);
            return Err(Box::new(CustomError(format!("{:?}", e))));
        }
    };
    let mut vs = model::Component::new(
        image::open(vs_path)?.resize(150, 150, imageops::FilterType::Nearest),
        None,
        None,
        Some("versus"),
    );
    vs.set_center_x(img.width);
    vs.set_center_y(img.height);

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
        model::Text::new(
            player1.get_str("discord_name").unwrap(),
            FONT_SIZE,
            0xFFFFFFFF,
            None,
        )
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
            player2.get_str("discord_name").unwrap(),
            FONT_SIZE,
            0xFFFFFFFF,
            None,
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
        model::Text::new(player1.get_str("tag").unwrap(), FONT_SIZE, 0xFFFFFFFF, None)
            .build()
            .await?,
        None,
        None,
        Some("tag1"),
    );
    tag1.set_relative_center_x(&name1);
    tag1.set_y(name1.y + name1.height() + 10);

    let mut tag2 = model::Component::new(
        model::Text::new(player2.get_str("tag").unwrap(), FONT_SIZE, 0xFFFFFFFF, None)
            .build()
            .await?,
        None,
        None,
        Some("tag2"),
    );
    tag2.set_relative_center_x(&name2);
    tag2.set_y(name2.y + name2.height() + 10);

    // Create text images for title, mode, VS, and footer
    let mut title_box = model::Component::new(
        model::Trapezoid {
            top: 200,
            bottom: 150,
            height: 100,
            color: 0xFFBF00FF,
            border: None,
        }
        .build()
        .await?
        .rotate180(),
        None,
        None,
        Some("title_box"),
    );
    title_box.set_center_x(img.width);
    title_box.set_y(img.height - title_box.height());
    let mut title_icon = model::Component::new(
        model::CustomImage::new(
            "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9ha3o5dFVFaWdrNWhMbWdWaFlHei5wbmcifQ:supercell:jcXu95iX7mdOU5lxdXU2Da8QR2BuK3rCgZgc_CwxcjU?width=2400",
            Some(100),
            Some(100),
        )
        .build()
        .await?,
        None,
        None,
        Some("title_icon"),
    );
    title_icon.set_center_x(title_box.width());
    title_icon.set_center_y(title_box.height());
    title_box.overlay(title_icon);
    let title_box_overlay = model::Component::new(
        model::Trapezoid {
            top: 200,
            bottom: 150,
            height: 100,
            color: 0xFFBF004D,
            border: Some(Border {
                thickness: 10,
                color: 0x000000FF,
            }),
        }
        .build()
        .await?
        .rotate180(),
        None,
        None,
        Some("title_overlay"),
    );
    title_box.overlay(title_box_overlay);

    let mut upper_title = model::Component::new(
        model::Text::new(
            format!("Round {round}"),
            35,
            0xFFFFFFFF,
            Some(model::Border {
                thickness: 3,
                color: 0x000000FF,
            }),
        )
        .build()
        .await?,
        None,
        None,
        Some("title"),
    );
    let mut lower_title = model::Component::new(
        model::Text::new(
            format!("Match {match_id}"),
            35,
            0xFFFFFFFF,
            Some(model::Border {
                thickness: 3,
                color: 0x000000FF,
            }),
        )
        .build()
        .await?,
        None,
        None,
        Some("title"),
    );
    upper_title.set_center_x(title_box.width());
    lower_title.set_center_x(title_box.width());
    let a = (title_box.height() - 2 * upper_title.height() - 5) / 2;
    upper_title.set_y(a);
    lower_title.set_y(upper_title.y + upper_title.height() + 5);
    title_box.overlay(upper_title);
    title_box.overlay(lower_title);
    // Create mode components
    let mut mode_bg = model::Component::new(
        model::Trapezoid {
            top: 300,
            bottom: 150,
            height: 75,
            color: 0xFE1AB6FF,
            border: {
                Some(model::Border {
                    thickness: 10,
                    color: 0x000000FF,
                })
            },
        }
        .build()
        .await?,
        None,
        None,
        Some("mode_bg"),
    );
    mode_bg.set_center_x(img.width);

    let mut mode_icon = model::Component::new(
        model::CustomImage::new(get_mode_icon(mode.to_string()), Some(150), Some(150))
            .build()
            .await?,
        None,
        None,
        Some("mode_icon"),
    );
    mode_icon.set_center_x(mode_bg.width());
    mode_icon.set_center_y(mode_bg.height());
    mode_bg.overlay(mode_icon);

    let mode_overlay = model::Component::new(
        model::Trapezoid {
            top: 300,
            bottom: 150,
            height: 75,
            color: 0xFE1AB64D,
            border: {
                Some(model::Border {
                    thickness: 10,
                    color: 0x000000FF,
                })
            },
        }
        .build()
        .await?,
        None,
        None,
        Some("mode_comp"),
    );
    mode_bg.overlay(mode_overlay);

    let mut mode_text = model::Component::new(
        model::Text::new(
            mode.to_uppercase(),
            40,
            0xFFFFFFFF,
            Some(model::Border {
                thickness: 3,
                color: 0x000000FF,
            }),
        )
        .build()
        .await?,
        None,
        None,
        Some("mode_text"),
    );
    mode_text.set_center_x(mode_bg.width());
    mode_text.set_center_y(mode_bg.height());
    mode_bg.overlay(mode_text);

    // Component elements onto the base img
    img.add_overlay(title_box);
    img.add_overlay(mode_bg);
    img.add_overlay(icon1);
    img.add_overlay(icon2);
    img.add_overlay(name1);
    img.add_overlay(name2);
    img.add_overlay(tag1);
    img.add_overlay(tag2);
    img.add_overlay(vs);
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

pub async fn get_image(
    player1: &Document,
    player2: &Document,
    config: &Document,
) -> Result<Vec<u8>, Error> {
    let img = generate_pre_battle_img(player1, player2, config).await?;
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
    Ok(bytes)
}
