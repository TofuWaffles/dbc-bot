use mongodb::bson::{doc, Document};
use tracing::{info, instrument};

use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};
use crate::checks::user_is_manager;
use crate::database_utils::find_round::get_round;
use crate::{Context, Error};
#[instrument]
#[poise::command(slash_command, guild_only)]
pub async fn disqualify(
    ctx: Context<'_>,
    #[description = "Select the user to disqualify"] user_id: u64,
    region: Region,
) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Attempting to disqualify player...")
        })
        .await?;

    info!("Attempting to disqualify player");
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    let round_collection = get_round(&config);
    let round = config.get("round").unwrap().as_i32().unwrap();
    let collection = ctx
        .data()
        .database
        .regional_databases
        .get(&region)
        .unwrap()
        .collection::<Document>(round_collection.as_str());

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
            msg.edit(ctx,|s|
                s.content(format!("Sucessfully disqualified player: {}({}) with respective Discord <@{}> at round {}", 
                    player.get("name").unwrap().as_str().unwrap(), 
                    player.get("tag").unwrap().as_str().unwrap(),
                    user_id,
                    round))
            )
            .await?;

            info!("Sucessfully disqualified player {}", user_id);
            Ok(())
        }
        None => {
            msg.edit(ctx, |s| {
                s.content(format!("No player is found for this ID at round {}", round))
            })
            .await?;
            Ok(())
        }
    }
}
