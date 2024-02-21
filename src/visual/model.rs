use crate::Error;
use async_trait::async_trait;
use bytes::Bytes;
use image::{
    imageops::{self, FilterType::Lanczos3},
    DynamicImage, ImageBuffer, Rgba,
};
use text_to_png::TextRenderer;
use tracing::error;
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
}

#[async_trait]
impl Image for Rectangle {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let (r, g, b, a) = get_color(self.color);

        // Create a new image with the specified dimensions
        let mut overlay_image = ImageBuffer::new(self.width as u32, self.height as u32);
        let bcolor = self.border.as_mut().map_or(0x00000000, |b| b.color);
        let (br, bg, bb, ba) = get_color(bcolor);

        // Fill the image with the provided RGBA color
        for (x, y, pixel) in overlay_image.enumerate_pixels_mut() {
            if Rectangle::is_border(&mut self, x as i64, y as i64){
                *pixel = Rgba([br, bg, bb, ba]);
            }
            else{
                *pixel = Rgba([r, g, b, a]);
            }
        }

        // Convert the ImageBuffer to a DynamicImage
        Ok(DynamicImage::ImageRgba8(overlay_image))
    }
}
impl Borderable for Rectangle {
    fn is_border(&mut self, x: i64, y: i64) -> bool {
        let t = self.border.as_mut().map_or(0, |t| t.thickness);
        x < t || x > self.width - t || y < t || y > self.height - t
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

        // Fill the image with the provided RGBA color
        for (x, y, pixel) in overlay_image.enumerate_pixels_mut() {
            if Trapezoid::is_inside(self, x as i64, y as i64) {
                *pixel = Rgba([r, g, b, a]);
            } else if Trapezoid::is_border(&mut self, x as i64, y as i64) {
                *pixel = Rgba([br, bg, bb, ba]);
            }
        }

        // Convert the ImageBuffer to a DynamicImage
        Ok(DynamicImage::ImageRgba8(overlay_image))
    }
}

impl Trapezoid {
    ///  Here is the geometry that Doofus doesn't like lmao
    ///    (0, 0)_______________________(top, 0)
    ///         \                      |
    ///          \                    |
    ///           \                  |
    /// (c,height) \________________| (bottom+c, height)
    ///
    pub fn is_inside(&mut self, x: i64, y: i64) -> bool {
        let c = ((self.top - self.bottom) / 2).abs();
        let t = self.border.as_mut().map_or(0, |t| t.thickness);
        self.height * (x - t) - c * y >= 0
            && y >= t
            && y <= self.height - t
            && self.height * (x - self.top + t) - y * (self.bottom + c - self.top) <= 0
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
}

#[async_trait]
impl Image for Text {
    async fn build(&mut self) -> Result<DynamicImage, Error> {
        let renderer = TextRenderer::try_new_with_ttf_font_data(include_bytes!(
            "./assets/LilitaOne-Regular.ttf"
        ))?;
        let img = renderer
            .render_text_to_png_data(self.text.clone(), self.font_size, self.font_color)?;
        
        match image::load_from_memory(&img.data) {
            Ok(img) => Ok(img),
            Err(e) => {
                error!("{e}");
                return Err(Error::from(e));
            }
        }
    }
}

impl Text {
    pub fn new<S>(text: S, font_size: u8, font_color: u32) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            font_size,
            font_color,
        }
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
    pub fn overlay(&mut self, overlay: Component) {
        imageops::overlay(&mut self.img, &overlay.img, overlay.x, overlay.y);
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
    let r = (color >> 24) as u8;
    let g = (color >> 16) as u8;
    let b = (color >> 8) as u8;
    let a = (color << 24) >> 24 as u8;
    (r as u8, g as u8, b as u8, a as u8)
}
