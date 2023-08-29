use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Ping the bot and possibly get a response, probably, maybe, maybe not?
#[poise::command(slash_command)]
pub async fn draco(ctx: Context<'_>) -> Result<(), Error> {
    let button_id = ctx.id();
    ctx.send(|s| {
        s.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.style(serenity::ButtonStyle::Success)
                        .custom_id(button_id)
                        .label("Yes")
                })
                .create_button(|b| {
                    b.style(serenity::ButtonStyle::Danger)
                        .custom_id("lol")
                        .label("No")
                        .disabled(true)
                })
            })
        })
        .reply(true)
        .ephemeral(false)
        .embed(|e| {
            e.title("**Do you love Draco (draco6)?**")
                .description("Answer carefully 😏")
        })
    })
    .await?;

    while let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == button_id.to_string())
        .await
    {
        let mut prompt = mci.message.clone();
        prompt.edit(ctx,|s| {
            s.components(|s| s)
              .embed(|e| {
                e.title("**Correct! Everyone loves Draco (draco6)**")
                    .description("Here is a cute animation")
                    .image("https://media.tenor.com/PHzgHGw2SHIAAAAC/dragon-fire.gif")
            })
        })
        .await?;
    }
    Ok(())
}
