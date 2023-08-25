pub mod api_handlers {
    use reqwest;
    use std::error::Error;
    use std::fmt;
    use serde_json;

    // Define a custom error type for your application
    #[derive(Debug)]
    struct CustomError(String);
    /// Implements the `fmt::Display` trait for the `CustomError` struct.
    ///
    /// This implementation allows instances of the `CustomError` struct to be formatted as strings
    /// when using the `format!`, `println!`, or `write!` macros. It displays the error message
    /// contained within the `CustomError` struct.
    ///
    /// # Example
    ///
    /// ```
    /// use your_crate_name::CustomError;
    ///
    /// let error = CustomError("An error occurred".to_string());
    /// println!("Error: {}", error); // Prints "Error: CustomError: An error occurred"
    /// ```
    impl fmt::Display for CustomError {
        /// Formats the `CustomError` instance as a string.
        ///
        /// # Arguments
        ///
        /// * `self` - The `CustomError` instance to format.
        /// * `f` - The formatter used to write the formatted output.
        ///
        /// # Returns
        ///
        /// A `fmt::Result` indicating whether the formatting operation was successful.
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "CustomError: {}", self.0)
        }
    }

    impl Error for CustomError {}

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
        match option {
            "player" => format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", tag),
            "battle_log" => format!("https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog", tag),
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
    ///             eprintln!("Error: {}", err);
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn request(endpoint: &str) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let token = std::env::var("BRAWL_STARS_TOKEN").expect("There is no Brawl Stars Token");
        let response = reqwest::Client::new()
            .get(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let data = response.json::<serde_json::Value>().await?;
            Ok(data)
        } else {
            eprintln!(
                "API responded with an unsuccessful status code: {}",
                response.status()
            );
            eprintln!("API response body: {:?}", response.text().await);
            Err(Box::new(CustomError("Unsuccessful response".to_string())))
        }
    }

}
