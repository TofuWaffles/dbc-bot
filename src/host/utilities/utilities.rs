use crate::{Context, Error};
use dbc_bot::Region;
use futures::StreamExt;
use poise::ReplyHandle;

use super::{announcement::announcement, config::configurate};
const TIMEOUT: u64 = 300;
pub async fn utilities_mod_panel(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title(format!("Utilities menu ({})", region.to_string()))
                .description(
                    r#"
        Announcement: Set an announcement for the tournament.
        Configurate: Set the configuration for the tournament.
        "#,
                )
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("announcement")
                        .label("Announcement")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                })
                .create_button(|b| {
                    b.custom_id("setting")
                        .label("Configurate")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
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
            _ => {}
        }
    }
    Ok(())
}
