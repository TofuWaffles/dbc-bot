use crate::brawlstars::api::{self, APIResult};
use crate::brawlstars::player::stat;
use crate::discord::prompt;
use crate::players::view::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::bson::Document;
use poise::ReplyHandle;

pub async fn view_info(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    player: Document,
) -> Result<(), Error> {
    let tag = player.get_str("tag").unwrap();
    let region = Region::find_key(player.get_str("region").unwrap()).unwrap();
    match api::request("player", tag).await {
        Ok(APIResult::Successful(player)) => {
            msg.edit(*ctx, |s|{
                s.components(|c|c)
                .embed(|e|e.description("Hold on..."))
            }).await?;
            stat(ctx, msg, &player, &region).await?;
        }
        Ok(APIResult::APIError(_)) => {
            prompt(
                ctx,
                msg,
                "The API is so uncanny!",
                "Please try again later",
                None,
                None,
            )
            .await?;
        }
        Ok(APIResult::NotFound(_)) | Err(_) => {
            prompt(
                ctx,
                msg,
                "Failed to find your account!",
                "We failed to find your account! Please try again!",
                None,
                None,
            )
            .await?;
        }
    };
    Ok(())
}
