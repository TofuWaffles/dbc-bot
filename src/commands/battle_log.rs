use crate::{Context, Error};
use crate::bracket_tournament::api;
use crate::utils::mode::get_mode_icon;
use crate::utils::embed_color::get_color;


/// Get the latest log of a player
#[poise::command(slash_command, prefix_command)]
pub async fn latest_log(
  ctx: Context<'_>, 
  #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), Error>{

  let endpoint = api::api_handlers::get_api_link("battle_log", &tag);
  match api::api_handlers::request(&endpoint).await{
    Ok(log) => {
      let player_endpoint = api::api_handlers::get_api_link("player", &tag);
      let player = api::api_handlers::request(&player_endpoint).await.unwrap();
      ctx.channel_id()
        .send_message(&ctx, |response|{
          response
            .allowed_mentions(|a| a.replied_user(true))
            .embed(|e|{
              e.title(format!("{}'s most recent match: {}",player["name"].to_string().strip_quote(),log["items"][0]["battle"]["result"].to_string().strip_quote()))
               .color(get_color(log["items"][0]["battle"]["result"].to_string().strip_quote()))
               .thumbnail(get_mode_icon().get(log["items"][0]["event"]["mode"].to_string().strip_quote().as_str()).unwrap())
               .field("Battle Time", log["items"][0]["battleTime"].to_string(),false)
               .fields(vec![
                ("Mode",log["items"][0]["event"]["mode"].to_string().strip_quote() ,true),
                ("Map",log["items"][0]["event"]["map"].to_string().strip_quote(),true),
                ("Duration",log["items"][0]["battle"]["duration"].to_string().strip_quote()+"s",true),
                ("Game",log["items"][0]["battle"]["type"].to_string().strip_quote(),true),
                ("Trophy Change",log["items"][0]["battle"]["trophyChange"].to_string().strip_quote(),true),
                ("","".to_string(),false)])

                .field("===============================".to_string(),"",false)
                .fields(vec![
                  (log["items"][0]["battle"]["teams"][0][0]["name"].to_string().strip_quote(),
                    "Brawler: ".to_string()+&log["items"][0]["battle"]["teams"][0][0]["brawler"]["name"].to_string().strip_quote()+"\n"
                    +"Power: "+&log["items"][0]["battle"]["teams"][0][0]["brawler"]["power"].to_string().strip_quote()
                  ,true),
                  ("VS".to_string(),"".to_string(),true),
                  (log["items"][0]["battle"]["teams"][1][0]["name"].to_string().strip_quote(),
                    "Brawler: ".to_string()+&log["items"][0]["battle"]["teams"][1][0]["brawler"]["name"].to_string().strip_quote()+"\n"
                    +"Power: "+&log["items"][0]["battle"]["teams"][1][0]["brawler"]["power"].to_string().strip_quote()
                  ,true),
                ])
        })
            }).await?;
    },
    Err(err) => {
      ctx.channel_id()
        .send_message(&ctx, |response|{
          response
            .allowed_mentions(|a| a.replied_user(true))
            .embed(|e|{
              e.title(format!("Error: {:#?}", err))
               .description(format!("No player is associated with {}", tag))
            })
      }).await?;
    }
  }

  Ok(())
}





/// A trait for stripping quotes from a string.
trait QuoteStripper {
  /// Strip double quotes from the string and return a new String.
  fn strip_quote(&self) -> String;
}

impl QuoteStripper for String {
  /// Strip double quotes from the string and return a new String.
  ///
  /// # Examples
  ///
  /// ```
  /// let s = String::from("\"Hello, world!\"");
  /// let stripped = s.strip_quote();
  /// assert_eq!(stripped, "Hello, world!");
  /// ```
  fn strip_quote(&self) -> String {
      let mut result = String::new();

      for c in self.chars() {
          if c != '"' {
              result.push(c);
          }
      }

      result
  }
}