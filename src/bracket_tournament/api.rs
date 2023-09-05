use crate::{
    misc::CustomError,
    Error
};
use poise::serenity_prelude::json::Value;
use reqwest;
use tracing::error;

/// Constructs an API link based on the provided option and tag.
///
/// # Arguments
///
/// * `option` - A string specifying the API endpoint option. Valid values are "player" and "battle_log".
/// * `tag` - A string representing the player's tag to be included in the URL.
///
/// # Panics
///
/// This function will panic if `option` is not "player" or "battle_log".
///
/// # Examples
///
/// ```
/// let player_tag = "ABC123";
/// let player_link = get_api_link("player", player_tag);
/// assert_eq!(player_link, "https://bsproxy.royaleapi.dev/v1/players/%23ABC123");
///
/// let battle_log_tag = "XYZ789";
/// let battle_log_link = get_api_link("battle_log", battle_log_tag);
/// assert_eq!(battle_log_link, "https://bsproxy.royaleapi.dev/v1/players/%23XYZ789/battlelog");
/// ```
pub fn get_api_link(option: &str, tag: &str) -> String {
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };

    match option {
        "player" => format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", proper_tag),
        "battle_log" => format!(
            "https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog",
            proper_tag
        ),
        "club" => format!(
            "https://bsproxy.royaleapi.dev/v1/clubs/%23{}/members",
            proper_tag
        ),
        _ => panic!("Unknown option"),
    }
}

/// Makes an asynchronous HTTP GET request to the specified endpoint with authentication.
///
/// This function takes an `endpoint` parameter, which is a string representing the URL
/// to which the HTTP GET request will be sent. It also expects the `BRAWL_STARS_TOKEN`
/// environment variable to be set, as it uses this token for authentication in the
/// request headers.
///
/// # Arguments
///
/// * `endpoint` - A string containing the URL to the API endpoint.
///
/// # Returns
///
/// A `Result` with the following possible outcomes:
///
/// - `Ok(serde_json::Value)` if the HTTP request is successful and the response
///    can be parsed as JSON.
/// - `Err(Box<dyn std::error::Error + Send + Sync>)` if there is an error in making
///    the request or parsing the response, or if the response status code indicates
///    failure.
///
/// # Errors
///
/// If the HTTP request returns a non-successful status code, an error message will
/// be printed to the standard error stream, and an `Err` variant will be returned
/// with a `CustomError` containing a descriptive message.
///
/// # Example
///
/// ```
/// use your_crate_name::request;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let endpoint = "https://example.com/api/data";
///     match request(endpoint).await {
///         Ok(data) => {
///             // Process the JSON data
///             println!("Received data: {:?}", data);
///         }
///         Err(err) => {
///             // Handle the error
///             error!("Error: {}", err);
///         }
///     }
///
///     Ok(())
/// }
/// ```
pub async fn request(endpoint: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let token = std::env::var("BRAWL_STARS_TOKEN").expect("Brawl Stars API token not found.");
    let response = reqwest::Client::new()
        .get(endpoint)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let data: Value = response.json().await?;
        Ok(data)
    } else {
        error!(
            "API responded with an unsuccessful status code: {}",
            response.status()
        );
        error!("API response body: {:?}", response.text().await);
        Err(Box::new(CustomError("Unsuccessful response".to_string())))
    }
}
