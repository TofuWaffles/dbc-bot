use crate::misc::CustomError;
use crate::Error;
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

fn get_player(player_tag: &str) -> String {
    format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", player_tag)
}

fn get_battle_log(player_tag: &str) -> String {
    format!(
        "https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog",
        player_tag
    )
}

/// Makes an asynchronous HTTP request to the Brawl Stars API.
///
/// This function sends a request to a specified API endpoint based on the provided
/// `option` and `tag` parameters. It includes an authorization header with a Bearer token
/// retrieved from the `BRAWL_STARS_TOKEN` environment variable.
///
/// # Arguments
///
/// * `option` - A string indicating the API endpoint option, such as "player" or "battle_log".
/// * `tag` - A string representing a tag or identifier used in the API request.
///
/// # Returns
///
/// * `Result<Value, Box<dyn std::error::Error + Send + Sync>>` - A result containing the JSON
///   response data if the request is successful, or an error if the request fails.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `Box<dyn std::error::Error + Send + Sync>` - A generic error type for any errors that occur.
/// * `CustomError` - An error indicating that the API response was unsuccessful.
///
/// # Panics
///
/// This function will panic if the `BRAWL_STARS_TOKEN` environment variable is not found.
///
/// # Examples
///
/// ```rust
/// use serde_json::Value;
/// use your_module::request;
///
/// async fn example() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let response = request("player", "#example_tag").await?;
///     // Process the JSON response data here
///     Ok(())
/// }
/// ```

pub async fn request(
    option: &str,
    tag: &str,
) -> Result<Value, Error> {
    let proper_tag = match tag.starts_with('#') {
        true => &tag[1..],
        false => tag,
    };
    let endpoint = match option {
        "player" => get_player(proper_tag),
        "battle_log" => get_battle_log(proper_tag),
        _ => unreachable!("Invalid option"),
    };

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
