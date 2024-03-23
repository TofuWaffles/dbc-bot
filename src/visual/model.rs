use crate::Error;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use bytes::Bytes;
use image::io::Reader as ImageReader;
use image::{
    imageops::{self, FilterType::Lanczos3},
    DynamicImage, ImageBuffer, Rgba,
};
use image::{GenericImage, GenericImageView, Pixel};

use std::env;
use std::io::{Cursor, Read};
use std::process::{Command, Stdio};
use tracing::{error, info};
const DEFAULT_ICON: &str = "https://cdn.brawlify.com/profile/28000000.png?v=1";
const DEFAULT_MODE_ICON: &str =
    "https://pbs.twimg.com/media/F2_Uy9rXgAAXXnP?format=png&name=360x360";

pub struct BSImage {
    pub width: i64,
    pub height: i64,
    pub bg: DynamicImage,
    pub name: String,
    pub overlay: Vec<Component>,
}
pub struct Component {
    pub x: i64,
    pub y: i64,
    pub img: DynamicImage,
    pub name: String,
}
pub struct Text {
    pub text: String,
    pub font_size: u8,
    pub font_color: u32,
    pub outline: Option<Border>,
}
pub struct Rectangle {
    pub width: i64,
    pub height: i64,
    pub color: u32,
    pub border: Option<Border>,
}

pub struct Trapezoid {
    pub top: i64,
    pub bottom: i64,
    pub height: i64,
    pub color: u32,
    pub border: Option<Border>,
}

pub struct Circle {
    pub radius: i64,
    pub color: u32,
    pub border: Option<Border>,
}
pub struct Border {
    pub thickness: i64,
    pub color: u32,
}
pub struct CustomImage {
    pub path: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[async_trait]
pub trait Image {
    async fn build(&mut self) -> Result<DynamicImage, Error>;
}

pub trait Borderable {
    fn is_border(&mut self, x: i64, y: i64) -> bool;
    fn is_inside(&mut self, x: i64, y: i64) -> bool;
    fn draw(
        &mut self,
        overlay_image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
        (r, g, b, a): (u8, u8, u8, u8),
        (br, bg, bb, ba): (u8, u8, u8, u8),
    ) -> DynamicImage {
        for (x, y, pixel) in overlay_image.enumerate_pixels_mut() {
            if self.is_inside(x as i64, y as i64) {
                *pixel = Rgba([r, g, b, a]);
            } else if self.is_border(x as i64, y as i64) {
                *pixel = Rgba([br, bg, bb, ba]);
            }
        }
        DynamicImage::ImageRgba8(overlay_image.clone())
    }
}

#[async_trait]
impl Image for Rectangle {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let color = get_color(self.color);
        let bcolor = self.border.as_mut().map_or(0x00000000, |b| b.color);
        let b_color = get_color(bcolor);
        let mut overlay_image: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(self.width as u32, self.height as u32);
        Ok(self.draw(&mut overlay_image, color, b_color))
    }
}
impl Borderable for Rectangle {
    fn is_border(&mut self, x: i64, y: i64) -> bool {
        let t = self.border.as_mut().map_or(0, |t| t.thickness);
        x < t || x > self.width - t || y < t || y > self.height - t
    }

    fn is_inside(&mut self, x: i64, y: i64) -> bool {
        x >= 0 && x <= self.width && y >= 0 && y <= self.height
    }
}

#[async_trait]
impl Image for Trapezoid {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let (r, g, b, a) = get_color(self.color);
        let bcolor = self.border.as_mut().map_or(0x00000000, |b| b.color);
        let (br, bg, bb, ba) = get_color(bcolor);
        // Create a new image with the specified dimensions
        let mut overlay_image = ImageBuffer::new(self.top as u32, self.height as u32);
        Ok(self.draw(&mut overlay_image, (r, g, b, a), (br, bg, bb, ba)))
    }
}

impl Borderable for Trapezoid {
    fn is_border(&mut self, x: i64, y: i64) -> bool {
        let c = ((self.top - self.bottom) / 2).abs();
        self.height * x - c * y >= 0
            && y >= 0
            && y <= self.height
            && self.height * (x - self.top) - y * (self.bottom + c - self.top) <= 0
    }
    //  Here is the geometry that Doofus doesn't like lmao
    //    (0, 0)_______________________(top, 0)
    //         \                      |
    //          \                    |
    //           \                  |
    // (c,height) \________________| (bottom+c, height)
    //
    fn is_inside(&mut self, x: i64, y: i64) -> bool {
        let c = ((self.top - self.bottom) / 2).abs();
        let t = self.border.as_mut().map_or(0, |t| t.thickness);
        self.height * (x - t) - c * y >= 0
            && y >= t
            && y <= self.height - t
            && self.height * (x - self.top + t) - y * (self.bottom + c - self.top) <= 0
    }
}
#[async_trait]
impl Image for Circle {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let (r, g, b, a) = get_color(self.color);
        let bcolor = self.border.as_mut().map_or(0x00000000, |b| b.color);
        let (br, bg, bb, ba) = get_color(bcolor);
        let mut overlay_image = ImageBuffer::new(self.radius as u32, self.radius as u32);
        Ok(self.draw(&mut overlay_image, (r, g, b, a), (br, bg, bb, ba)))
    }
}

impl Borderable for Circle {
    fn is_border(&mut self, x: i64, y: i64) -> bool {
        x * x + y * y <= self.radius * self.radius
    }

    fn is_inside(&mut self, x: i64, y: i64) -> bool {
        let t = self.border.as_mut().map_or(0, |t| t.thickness);
        x * x + y * y <= (self.radius - t) * (self.radius - t)
    }
}
#[async_trait]
impl Image for Text {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        self.generate_text_img()
    }
}

impl Text {
    pub fn new<S>(text: S, font_size: u8, font_color: u32, outline: Option<Border>) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            font_size,
            font_color,
            outline,
        }
    }
    fn generate_text_img(&self) -> Result<DynamicImage, Error> {
        let (stroke, stroke_color) = self
            .outline
            .as_ref()
            .map_or_else(|| (0, 0x00000000_u32), |b| (b.thickness, b.color));
        let data = format!(
            r#"{{
                "text": "{text}",
                "font_size": {font_size},
                "font_color": "{font_color}",
                "stroke_width": "{stroke}",
                "stroke_color": "{stroke_color}"
            }}"#,
            text = self.text,
            font_size = self.font_size,
            font_color = self.font_color
        );
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                error!("Failed to get current directory: {e}");
                return Err(e.into());
            }
        };
        let output = Command::new("python3")
            .arg("scripts/text.py")
            .arg(data)
            .stdout(Stdio::piped())
            .current_dir(current_dir)
            .spawn()?;

        let stdout = output.wait_with_output()?.stdout;
        let buffer = std::str::from_utf8(&stdout)?;
        if buffer.len() < 100 {
            return Err("Failed to capture Python script output".into());
        }
        let image_bytes = match general_purpose::STANDARD.decode(buffer.trim_end()) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("{e}");
                info!("Debug: {buffer}");
                return Err(e.into());
            }
        };
        let img = ImageReader::new(Cursor::new(image_bytes))
            .with_guessed_format()?
            .decode()?;
        Ok(img)
    }
}

impl CustomImage {
    pub fn new<S>(path: S, width: Option<i64>, height: Option<i64>) -> Self
    where
        S: Into<String>,
    {
        Self {
            path: path.into(),
            width,
            height,
        }
    }
}

#[async_trait]
impl Image for CustomImage {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let img_bytes = match reqwest::get(&self.path).await {
            Ok(res) => {
                if res.status() == 200 {
                    res.bytes().await?
                } else {
                    error!("Failed to fetch image from {}", self.path);
                    self.default_image_bytes().await?
                }
            }
            Err(e) => {
                error!("{e}");
                self.default_image_bytes().await?
            }
        };
        match (self.width, self.height) {
            (Some(width), Some(height)) => {
                let img = image::load_from_memory(&img_bytes)?.resize_exact(
                    width as u32,
                    height as u32,
                    Lanczos3,
                );
                Ok(img)
            }
            (_, _) => {
                let img = image::load_from_memory(&img_bytes)?;
                self.width = Some(img.width() as i64);
                self.height = Some(img.height() as i64);
                Ok(img)
            }
        }
    }
}

impl CustomImage {
    async fn default_image_bytes(&self) -> Result<Bytes, Error> {
        let icon = if self.path.contains("profile") {
            reqwest::get(DEFAULT_ICON).await?.bytes().await?
        } else if self.path.contains("event") {
            reqwest::get(DEFAULT_MODE_ICON).await?.bytes().await?
        } else {
            unreachable!("Invalid icon type")
        };
        Ok(icon)
    }
}
#[allow(dead_code)]
impl Component {
    pub fn new<S>(img: DynamicImage, x: Option<i64>, y: Option<i64>, name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        Self {
            img: img.clone(),
            x: x.unwrap_or(0),
            y: y.unwrap_or(0),
            name: name.map_or_else(|| "untitled".to_string(), |s| s.into()),
        }
    }
    pub fn width(&self) -> i64 {
        self.img.width() as i64
    }

    pub fn height(&self) -> i64 {
        self.img.height() as i64
    }
    pub fn set_x(&mut self, x: i64) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: i64) {
        self.y = y;
    }

    pub fn set_center_x(&mut self, parent_width: i64) {
        self.x = (parent_width - self.img.width() as i64) / 2;
    }

    pub fn set_center_y(&mut self, parent_height: i64) {
        self.y = (parent_height - self.img.height() as i64) / 2;
    }

    pub fn set_relative_center_x(&mut self, dependent: &Component) {
        self.set_center_x(2 * dependent.x + dependent.width());
    }

    pub fn set_relative_center_y(&mut self, dependent: &Component) {
        self.set_center_y(2 * dependent.y + dependent.height());
    }

    pub fn set_dimensions(&mut self, width: i64, height: i64) {
        self.img = self.img.resize_exact(width as u32, height as u32, Lanczos3);
    }
    /// Get the x coordinate to center this component on another component
    pub fn get_center_x(&self, base_width: i64) -> i64 {
        (base_width - self.img.width() as i64) / 2
    }

    /// Get the y coordinate to center this component on another component
    pub fn get_center_y(&self, base_height: &i64) -> i64 {
        (base_height - self.img.height() as i64) / 2
    }

    /// Overlay another component on this component
    ///
    /// I copy from the source code and add another conditions lol.
    pub fn overlay(&mut self, top: Component) {
        let top_dims = top.img.dimensions();

        // Crop our top image if we're going out of bounds
        let (
            origin_bottom_x,
            origin_bottom_y,
            origin_top_x,
            origin_top_y,
            range_width,
            range_height,
        ) = self.overlay_bounds_ext(top_dims, top.x, top.y);

        for y in 0..range_height {
            for x in 0..range_width {
                let p = top.img.get_pixel(origin_top_x + x, origin_top_y + y);
                let mut bottom_pixel = self.img.get_pixel(origin_bottom_x + x, origin_bottom_y + y);
                bottom_pixel.blend(&p);
                let place_x = origin_bottom_x + x;
                let place_y = origin_bottom_y + y;
                if self.img.get_pixel(place_x, place_y)[3] == 0 {
                    continue;
                } else {
                    self.img.put_pixel(place_x, place_y, bottom_pixel);
                }
            }
        }
    }

    fn overlay_bounds_ext(
        &self,
        (top_width, top_height): (u32, u32),
        x: i64,
        y: i64,
    ) -> (u32, u32, u32, u32, u32, u32) {
        let (bottom_width, bottom_height) = (self.width(), self.height());
        // Return a predictable value if the two images don't overlap at all.
        if x > bottom_width
            || y > bottom_height
            || x.saturating_add(top_width.into()) <= 0
            || y.saturating_add(top_height.into()) <= 0
        {
            return (0, 0, 0, 0, 0, 0);
        }

        // Find the maximum x and y coordinates in terms of the bottom image.
        let max_x = x.saturating_add(i64::from(top_width));
        let max_y = y.saturating_add(i64::from(top_height));

        // Clip the origin and maximum coordinates to the bounds of the bottom image.
        // Casting to a u32 is safe because both 0 and `bottom_{width,height}` fit
        // into 32-bits.
        let max_inbounds_x = max_x.clamp(0, bottom_width) as u32;
        let max_inbounds_y = max_y.clamp(0, bottom_height) as u32;
        let origin_bottom_x = x.clamp(0, bottom_width) as u32;
        let origin_bottom_y = y.clamp(0, bottom_height) as u32;

        // The range is the difference between the maximum inbounds coordinates and
        // the clipped origin. Unchecked subtraction is safe here because both are
        // always positive and `max_inbounds_{x,y}` >= `origin_{x,y}` due to
        // `top_{width,height}` being >= 0.
        let x_range = max_inbounds_x - origin_bottom_x;
        let y_range = max_inbounds_y - origin_bottom_y;

        // If x (or y) is negative, then the origin of the top image is shifted by -x (or -y).
        let origin_top_x = x.saturating_mul(-1).clamp(0, i64::from(top_width)) as u32;
        let origin_top_y = y.saturating_mul(-1).clamp(0, i64::from(top_height)) as u32;

        (
            origin_bottom_x,
            origin_bottom_y,
            origin_top_x,
            origin_top_y,
            x_range,
            y_range,
        )
    }
}

impl BSImage {
    pub fn new<S>(width: Option<i64>, height: Option<i64>, bg_path: String, name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let bg = image::open(bg_path).unwrap();
        match (width, height) {
            (Some(width), Some(height)) => Self {
                width,
                height,
                bg,
                name: name.map_or_else(|| "untitled".to_string(), |s| s.into()),
                overlay: vec![],
            },
            (_, _) => Self {
                width: bg.width() as i64,
                height: bg.height() as i64,
                bg,
                name: name.map_or_else(|| "untitled".to_string(), |s| s.into()),
                overlay: vec![],
            },
        }
    }

    pub fn add_overlay(&mut self, overlay: Component) {
        self.overlay.push(overlay);
    }

    pub fn build(&mut self) -> DynamicImage {
        for overlay in &self.overlay {
            imageops::overlay(&mut self.bg, &overlay.img, overlay.x, overlay.y);
        }
        self.bg.clone()
    }
}

fn get_color(color: u32) -> (u8, u8, u8, u8) {
    let r = color >> 24_u8;
    let g = color >> 16_u8;
    let b = color >> 8_u8;
    let a = (color << 24_u8) >> 24_u8;
    (r as u8, g as u8, b as u8, a as u8)
}
