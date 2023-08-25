use crate::{Context, Error};
use crate::bracket_tournament::api;

/// Get the player's profile
#[poise::command(slash_command, prefix_command)]
pub async fn player(
  ctx: Context<'_>, 
  #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), Error>{

  let endpoint = api::api_handlers::get_api_link("player", &tag);
  match api::api_handlers::request(&endpoint).await{
    Ok(player) => {
      ctx.channel_id()
        .send_message(&ctx, |response|{
          response
            .allowed_mentions(|a| a.replied_user(false))
            .embed(|e|{
              e.title(format!("**{}({})**",player["name"], player["tag"]))
                .thumbnail(format!("https://cdn-old.brawlify.com/profile-low/{}.png", player["icon"]["id"]))
                .fields(vec![
                  ("Trophies", player["trophies"].to_string(), true),
                  ("Highest Trophies", player["highestTrophies"].to_string(), true),
                  ("3v3 Victories",player["3vs3Victories"].to_string(), true),
                  ("Solo Victories", player["soloVictories"].to_string(), true),
                  ("Duo Victories", player["duoVictories"].to_string(), true),
                  ("Best Robo Rumble Time", player["bestRoboRumbleTime"].to_string(), true),
                  ("Club", player["club"]["name"].to_string(), true),
              ])
              .timestamp(ctx.created_at())
            })
      }).await?;
    },
    Err(_) => {
      ctx.channel_id()
        .send_message(&ctx, |response|{
          response
            .allowed_mentions(|a| a.replied_user(true))
            .embed(|e|{
              e.title(format!("**We have tried very hard to find but...**"))
               .description(format!("No player is associated with the tag #{}", tag))
            })
      }).await?;
    }
  }
  Ok(())
  }


