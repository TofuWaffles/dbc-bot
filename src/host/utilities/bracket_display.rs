use crate::{
    bracket_tournament::bracket_update::update_bracket, discord::prompt::prompt, Context, Error,
};
use dbc_bot::Region;
use poise::ReplyHandle;

pub async fn bracket_display(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Generating bracket image",
        "<a:loading:1187839622680690689> Please wait while the image is being generated",
        None,
        None,
    ).await?;
    update_bracket(ctx, Some(region)).await?;
    prompt(ctx, msg, "Bracket", "Bracket has been updated", None, None).await?;
    Ok(())
}
