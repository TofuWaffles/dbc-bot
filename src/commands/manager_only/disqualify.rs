use mongodb::bson::{doc, Document};
use tracing::{info, instrument};

use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};
use crate::checks::user_is_manager;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
#[instrument]
#[poise::command(slash_command, guild_only)]
pub async fn disqualify(
    ctx: Context<'_>,
    #[description = "The ID of the user to disqualify"] user_id: u64,
    region: Region,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    info!("Attempting to disqualify player");
    let collection = ctx
        .data()
        .database
        .regional_databases
        .get(&region)
        .unwrap()
        .collection::<Document>("Player");

    let player = collection
        .find_one(doc! {"discord_id": user_id.to_string()}, None)
        .await?;

    match player {
        Some(player) => {
            let match_id = player
                .get("match_id")
                .unwrap()
                .to_string()
                .parse::<i32>()
                .unwrap();
            let mannequin = add_mannequin(&region, Some(match_id), None);
            collection
                .delete_one(doc! {"discord_id": user_id.to_string()}, None)
                .await?;
            collection.insert_one(mannequin, None).await?;
            ctx.send(|s| {
                s.ephemeral(true)
                    .reply(true)
                    .content(format!("Sucessfully disqualified player: {}({}) with respective Discord <@{}> at round {}", 
                    player.get("name").unwrap().to_string().strip_quote(), 
                    player.get("tag").unwrap().to_string().strip_quote(),
                    user_id.to_string(),
                    round))
            })
            .await?;

            info!("Sucessfully disqualified player {}", user_id);
            return Ok(());
        }
        None => {
            ctx.send(|s|{
                s.content(format!("No player is found for this ID at round {}", round))
                .reply(true)
                .ephemeral(true)
            }).await?;
            return Ok(());
        }
    }
}
