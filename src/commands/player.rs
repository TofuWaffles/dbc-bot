use poise::CreateReply;
use poise::serenity_prelude::{Embed, CreateEmbed};

use crate::utils::types;
use crate::bracket_tournament::player::Player;

/// Get the player's profile
#[poise::command(slash_command, prefix_command)]
pub async fn player(
  ctx: types::Context<'_>, 
  #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), types::Error> {
  let player = Player::new(&tag).await;
  let embed = CreateReply{
    content: None,
    attachments: vec![],
    allowed_mentions: None,
    components: None,
    ephemeral: false,
    reply: false,
    embeds: vec![CreateEmbed()
      .title(format!("**{}({})**",player.name, player.tag))
      .fields(vec![
        ("Trophies", format!("{}", player.trophies), true),
        ("Highest Trophies", format!("{}", player.highest_trophies), true),
        ("3v3 Victories", format!("{}", player.victories_3v3), true),
        ("Solo Victories", format!("{}", player.solo_victories), true),
        ("Duo Victories", format!("{}", player.duo_victories), true),
        ("Best Robo Rumble Time", format!("{}", player.best_robo_rumble_time), true),
        ("Club", format!("{}({})", player.club.name, player.club.tag), true),
      ])
    ],
  };
  let response = |embed: &mut CreateReply| embed;
  ctx.send(response).await?;
  Ok(())
}

