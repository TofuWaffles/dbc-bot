
use crate::{bracket_tournament::bracket_update::update_bracket, Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;

pub async fn bracket_display(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e|{
            e.title("Image is generating")
            .description("<a:loading:1187839622680690689> Please wait while the image is being generated")
        })
    }).await?;
    update_bracket(ctx, Some(&region)).await?;
    msg.edit(*ctx, |m| m.embed(|e|e.title("Bracket updated"))).await?;
    Ok(())
}