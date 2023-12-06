use poise::ReplyHandle;

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
    title: &str,
    description: &str,
    image: Option<&str>,
    color: Option<u32>,
) -> Result<(), Error> {
    let c = match color {
        Some(c) => c,
        None => 0xFFFFFF,
    };
    msg.edit(*ctx, |b| {
        b.embed(|e| {
            e.title(title)
                .description(description)
                .color(c)
                .image(image.unwrap_or(""))
        })
    })
    .await?;
    Ok(())
}
