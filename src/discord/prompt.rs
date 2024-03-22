use poise::{serenity_prelude::Colour, ReplyHandle};

use crate::{Context, Error};

///For simple prompts, if complicated please use the library directly
/// # Arguments
/// * `ctx` - The context of the application.
/// * `msg` - The message to edit.
/// * `title` - The title of the embed.
/// * `description` - The description of the embed.
/// * `image` - The link to the image of the embed.
/// * `color` - The color of the embed.
pub async fn prompt(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    title: impl Into<String>,
    description: impl Into<String>,
    image: Option<&str>,
    color: impl Into<Option<u32>>,
) -> Result<(), Error> {
    msg.edit(*ctx, |b| {
        b.embed(|e| {
            e.title(title.into())
                .description(description.into())
                .color(Colour::new(color.into().unwrap_or(0)))
                .image(image.unwrap_or(""))
        })
        .components(|c| c)
    })
    .await?;
    Ok(())
}
