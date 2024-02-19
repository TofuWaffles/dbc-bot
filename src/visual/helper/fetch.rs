use bytes::Bytes;
use image::imageops::Lanczos3;
use image::DynamicImage;

/// Fetch and optionally resize an image from a URL.
///
/// This function fetches an image from the specified `url` and, if provided, resizes it to the
/// specified dimensions (`resize_width` and `resize_height`). If the URL fetch fails, it falls
/// back to a default image.
///
/// # Arguments
///
/// * `url` - A string containing the URL of the image to fetch.
/// * `(resize_width, resize_height)` - A tuple of optional values representing the desired
///   dimensions to resize the image. Use `None` for either dimension to maintain aspect ratio.
///
/// # Returns
///
/// * `DynamicImage` - A dynamic image loaded from the fetched or default image.
///
/// # Example
///
/// ```rust
/// use your_module::fetch_image;
///
/// #[tokio::main]
/// async fn main() {
///     let url = "https://example.com/image.jpg".to_string();
///     let image = fetch_image(url, (Some(200), Some(150))).await;
///     image.save("output.png").expect("Failed to save image");
/// }
/// ```
pub async fn fetch_image(
    url: String,
    (resize_width, resize_height): (Option<u32>, Option<u32>),
) -> DynamicImage {
    let img_bytes = match reqwest::get(url.clone()).await {
        Ok(res) => {
            if res.status() == 200 {
                res.bytes().await.unwrap()
            } else {
                println!("Failed to fetch image from {}", url);
                let icon: Bytes;
                if url.contains("profile") {
                    icon = Bytes::copy_from_slice(
                        &include_bytes!("../assets/default_player_icon.png")[..],
                    );
                } else if url.contains("event") {
                    icon = Bytes::copy_from_slice(
                        &include_bytes!("../assets/default_mode_icon.png")[..],
                    );
                } else {
                    unreachable!("Invalid icon type")
                }
                icon
            }
        }
        Err(_) => {
            println!("Failed to fetch image from {}", url);
            let icon: Bytes;
            if url.contains("profile") {
                icon = Bytes::copy_from_slice(
                    &include_bytes!("../assets/default_player_icon.png")[..],
                );
            } else if url.contains("event") {
                icon =
                    Bytes::copy_from_slice(&include_bytes!("../assets/default_mode_icon.png")[..]);
            } else {
                unreachable!("Invalid icon type")
            }
            icon
        }
    };

    let original = image::load_from_memory(&img_bytes).expect("Failed to load image");
    match (resize_width, resize_height) {
        (Some(width), Some(height)) => original.resize_exact(width, height, Lanczos3),
        (_, _) => original,
    }
}
