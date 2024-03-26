use crate::{discord::prompt::prompt, Context, Error};
use dbc_bot::Region;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::ReplyHandle;

use super::{
    config::get_config,
    find::{find_enemy_by_match_id_and_self_tag, find_round_from_config},
    update::update_result,
};

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

pub async fn force_lose(
    ctx: &Context<'_>,
    region: &Region,
    player: &Document,
    reason: &str,
) -> Result<(), Error> {
    let round = find_round_from_config(&get_config(ctx, region).await);
    let match_id = player.get_i32("match_id")?;
    let player_tag = player.get_str("tag")?;
    let opponent =
        match find_enemy_by_match_id_and_self_tag(ctx, region, &round, &match_id, player_tag).await
        {
            Some(opponent) => opponent,
            None => {
                return Err("No opponent found!".into());
            }
        };
    update_result(ctx, region, &round, &opponent, player, reason).await
}
