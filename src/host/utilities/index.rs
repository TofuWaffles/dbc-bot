use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use poise::{serenity_prelude::ReactionType, ReplyHandle};

use super::{
    announcement::announcement, bracket_display::bracket_display, config::configurate, test::test,
};
const TIMEOUT: u64 = 300;
pub async fn utilities_mod_panel(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title(format!("Utilities menu ({})", region.short()))
                .description(
                    r#"Below are available options:
📢: Announcement
- Set an announcement for the tournament.
🛠️: Configuration
- Set the configuration for the tournament.
🎾: Bracket
- Display the bracket for the tournament.
-🧪: Test
- Test sending to a channel.
"#,
                )
                .color(0xFFFF00)
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("announcement")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("📢".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("setting")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .emoji(ReactionType::Unicode("🛠️".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("bracket")
                        .style(poise::serenity_prelude::ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("🎾".to_string()))
                })
                .create_button(|b| {
                    b.custom_id("test")
                        .style(poise::serenity_prelude::ButtonStyle::Secondary)
                        .emoji(ReactionType::Unicode("🧪".to_string()))
                })
            })
        })
    })
    .await?;
    let cib = msg
        .clone()
        .into_message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "setting" => {
                mci.defer(&ctx.http()).await?;
                return configurate(ctx, msg, region).await;
            }
            "announcement" => {
                mci.defer(&ctx.http()).await?;
                return announcement(ctx, msg).await;
            }
            "bracket" => {
                mci.defer(&ctx.http()).await?;
                return bracket_display(ctx, msg, region).await;
            }
            "test" => {
                mci.defer(&ctx.http()).await?;
                return test(ctx, msg, region).await;
            }
            _ => {}
        }
    }
    Ok(())
}
