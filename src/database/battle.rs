use crate::{discord::prompt::prompt, Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::ReplyHandle;

pub async fn battle_happened(
    ctx: &Context<'_>,
    tag: &str,
    round: &Collection<Document>,
    msg: &ReplyHandle<'_>,
) -> Result<Option<mongodb::bson::Document>, Error> {
    let player = round.find_one(doc! {"tag": tag}, None).await?;
    match player {
        Some(player) => {
            if player
                .get("battle")
                .and_then(|b| b.as_bool())
                .unwrap_or(false)
            {
                msg.edit(*ctx, |s| s.content("You have already submitted the result! Please wait until the next round begins!")).await?;
                prompt(
                    ctx,
                    msg,
                    "You've already played this round!",
                    "Please wait until next round starts!",
                    None,
                    Some(0x00FF00),
                )
                .await?;
                Ok(None)
            } else {
                Ok(Some(player))
            }
        }
        None => {
            msg.edit(*ctx, |s| {
                s.content("You are not in this round! Oops! Better luck next time")
            })
            .await?;
            Ok(None)
        }
    }
}

pub async fn is_battle(ctx: &Context<'_>, tag: Option<&str>, round: String) -> Result<bool, Error> {
    let collection: mongodb::Collection<mongodb::bson::Document> =
        ctx.data().database.general.collection(round.as_str());
    let player = collection.find_one(doc! {"tag": tag}, None).await?;
    match player {
        Some(player) => {
            return Ok(player
                .get("battle")
                .and_then(|b| b.as_bool())
                .unwrap_or(false))
        }
        None => Ok(false),
    }
}
