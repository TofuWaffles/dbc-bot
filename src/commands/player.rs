use crate::bracket_tournament::player::Player;
use crate::{Context, Error};

/// Get the player's profile
#[poise::command(slash_command, prefix_command)]
pub async fn player(
    ctx: Context<'_>,
    #[description = "Put your tag here (without #)"] tag: String,
) -> Result<(), Error> {
    let player = Player::new(&tag).await;
    ctx.channel_id()
        .send_message(&ctx, |response| {
            response
                .allowed_mentions(|a| a.replied_user(true))
                .embed(|e| {
                    e.title(format!("**{}({})**", player.name, player.tag))
                        .thumbnail(format!(
                            "https://cdn-old.brawlify.com/profile-low/{}.png",
                            player.icon.id
                        ))
                        .fields(vec![
                            ("Trophies", format!("{}", player.trophies), true),
                            (
                                "Highest Trophies",
                                format!("{}", player.highest_trophies),
                                true,
                            ),
                            ("3v3 Victories", format!("{}", player.victories_3v3), true),
                            ("Solo Victories", format!("{}", player.solo_victories), true),
                            ("Duo Victories", format!("{}", player.duo_victories), true),
                            (
                                "Best Robo Rumble Time",
                                format!("{}", player.best_robo_rumble_time),
                                true,
                            ),
                            ("Club", format!("{}", player.club.name), true),
                        ])
                        .timestamp(ctx.created_at())
                })
        })
        .await?;
    Ok(())
}
