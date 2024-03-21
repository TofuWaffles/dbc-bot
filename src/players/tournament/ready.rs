use crate::{
    database::{
        config::get_config,
        find::{find_round_from_config, find_self_by_discord_id},
        update::set_ready,
    },
    discord::prompt::prompt,
    Context, Error,
};

use dbc_bot::Region;
use mongodb::bson::Document;
use poise::ReplyHandle;

pub async fn ready(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
    player: Document,
) -> Result<(), Error> {
    let round = find_round_from_config(&get_config(ctx, region).await);
    let discord_id = player.get_str("discord_id").unwrap();
    set_ready(ctx, region, &round, discord_id).await?;
    return prompt(
        ctx,
        msg,
        "Ready",
        "You have been marked as ready for the next round.",
        None,
        Some(0x00FF00),
    )
    .await;
}
