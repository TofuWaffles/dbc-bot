use crate::Region;
use crate::{Context, Error};
use poise::ReplyHandle;

pub async fn player_module(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    _region: &Region,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| e.title("Getting battle from this player..."))
    })
    .await?;
    Ok(())
}
