use crate::{Context, Error};
use mongodb::bson::doc;
use poise::ReplyHandle;

pub async fn battle_happened(
    ctx: &Context<'_>,
    tag: &str,
    round: mongodb::Collection<mongodb::bson::Document>,
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

pub async fn is_battle(
    ctx: &Context<'_>,
    tag: Option<&str>,
    round: String,
) -> Result<bool, Error> {
    let collection: mongodb::Collection<mongodb::bson::Document> = ctx.data().database.general.collection(round.as_str());
    let player = collection.find_one(doc! {"tag": tag}, None).await?;
    match player {
        Some(player) => {
            return Ok(player
                .get("battle")
                .and_then(|b| b.as_bool())
                .unwrap_or(false))
        }
        None => {
            return Ok(false);
        }
    }
}