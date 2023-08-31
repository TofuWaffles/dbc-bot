use crate::bracket_tournament::api;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
/// Get the club's profile
#[poise::command(slash_command, guild_only)]
pub async fn club(
    ctx: Context<'_>,
    #[description = "Put your club here (without #)"] tag: String,
) -> Result<(), Error> {
    let endpoint = api::get_api_link("club", &tag.to_uppercase());
    match api::request(&endpoint).await {
        Ok(club) => {
            let mut data = "".to_string();
            let club = club["items"].as_array().unwrap();
            for i in club {
                data += format!(
                    "name: {} tag: {} \n",
                    &i["name"].to_string().strip_quote(),
                    &i["tag"].to_string().strip_quote()
                )
                .as_str();
            }
            ctx.say(data).await?;
        }
        Err(_) => {}
    };
    Ok(())
}
