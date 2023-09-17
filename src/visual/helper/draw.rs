use image::{DynamicImage, ImageBuffer, Rgba};
use text_to_png::TextRenderer;

/// Creates a dynamic image with a filled rectangle of the specified dimensions and color.
///
/// This function generates a dynamic image with a filled rectangle of the specified `width`,
/// `height`, and `color`. The color should be represented as a u32 in the format 0xRRGGBBAA
/// (Red, Green, Blue, Alpha).
///
/// # Arguments
///
/// * `width` - The width of the rectangle in pixels.
/// * `height` - The height of the rectangle in pixels.
/// * `color` - The fill color of the rectangle, represented as a u32.
///
/// # Returns
///
/// * `DynamicImage` - A dynamic image containing the filled rectangle.
///
/// # Example
///
/// ```rust
/// let width = 200;
/// let height = 100;
/// let color = 0xFF336699; // RGBA color (0xRRGGBBAA)
/// let image = draw_rec(width, height, color);
/// image.save("output.png").expect("Failed to save image");
/// ```
pub fn draw_rec(width: u32, height: u32, color: u32) -> DynamicImage {
    // Extract the RGBA components from the u32 color
    let r = ((color >> 16) & 0xFF) as u8;
    let g = (((color << 8) >> 16) & 0xFF) as u8;
    let b = (((color << 16) >> 16) & 0xFF) as u8;
    let a = 0xFF;

    // Create a new image with the specified dimensions
    let mut overlay_image = ImageBuffer::new(width, height);

    // Fill the image with the provided RGBA color
    for (_, _, pixel) in overlay_image.enumerate_pixels_mut() {
        *pixel = Rgba([r, g, b, a]); // Pink color (R, G, B, A)
    }

    // Convert the ImageBuffer to a DynamicImage
    DynamicImage::ImageRgba8(overlay_image)
}
/// Generates a dynamic image containing the specified text.
///
/// This function renders the provided `text` using the LilitaOne-Regular font at the
/// specified `font_size` and `font_color`, and returns a `DynamicImage`.
///
/// # Arguments
///
/// * `text` - A string containing the text to render.
/// * `font_size` - The font size to use for rendering the text.
/// * `font_color` - The font color represented as a u32 in the format 0xRRGGBBAA (Red, Green,
///   Blue, Alpha).
///
/// # Returns
///
/// * `DynamicImage` - A dynamic image containing the rendered text.
///
/// # Panics
///
/// This function may panic if it fails to load the font or render the text.
///
/// # Example
///
/// ```rust
/// let text_image = make_text_image("Hello, Rust!", 24, &0xFF0000FF);
/// text_image.save("output.png").expect("Failed to save image");
/// ```
pub fn make_text_image(text: &str, font_size: u8, font_color: &u32) -> DynamicImage {
    let renderer = TextRenderer::try_new_with_ttf_font_data(
        include_bytes!("../asset/LilitaOne-Regular.ttf"),
    )
    .unwrap();
    let img = renderer
        .render_text_to_png_data(text, font_size, *font_color)
        .expect("Failed to render text");
    image::load_from_memory(&(img.data)).expect("Failed to load image")
}
