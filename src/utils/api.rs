pub mod api_handlers {
    use reqwest;
    use std::error::Error;
    use std::fmt;

    // Define a custom error type for your application
    #[derive(Debug)]
    struct CustomError(String);

    impl fmt::Display for CustomError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "CustomError: {}", self.0)
        }
    }

    impl Error for CustomError {}

    pub fn get_api_link(option: &str, tag: &str) -> String {
        match option {
            "player" => format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", tag),
            "battle_log" => format!(
                "https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog",
                tag
            ),
            _ => panic!("Unknown option"),
        }
    }

    pub async fn request<T>(endpoint: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        T: serde::de::DeserializeOwned, // Deserialize for JSON parsing
    {
        let token = std::env::var("BRAWL_STARS_TOKEN").expect("There is no Brawl Stars Token");
        let response = reqwest::Client::new()
            .get(endpoint)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        if response.status().is_success() {
            let data: T = response.json().await?;
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
