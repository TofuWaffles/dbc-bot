use mongodb::bson::{doc, Document};
use tracing::{info, instrument};

use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
#[instrument]
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn disqualify(
    ctx: Context<'_>,
    #[description = "The ID of the user to disqualify"] player: serenity::User,
    region: Region,
) -> Result<(), Error> {
    let user_id = player.id;
    info!("Attempting to disqualify player");
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config: Document = database
        .collection("Config")
        .find_one(None, None)
        .await
        .unwrap()
        .unwrap();
    let round = match config.get("round").unwrap().to_string().parse::<i32>() {
        Ok(round) => {
            if round == 0 {
                "Player".to_string()
            } else {
                format!("Round {}", round)
            }
        }
        Err(e) => {
            ctx.say(format!("Error occurred while parsing round number: {}", e))
                .await?;
            return Ok(());
        }
    };
    let collection = database.collection::<Document>(round.as_str());
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
