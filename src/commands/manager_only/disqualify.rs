// use mongodb::bson::{doc, Document};
// use crate::bracket_tournament::config::get_config;
// use crate::bracket_tournament::{mannequin::add_mannequin, region::Region};
// use crate::checks::user_is_manager;
// use crate::database_utils::find_round::get_round;
// use crate::{Context, Error};

// #[derive(Debug, poise::Modal)]
// #[name = "Disqualify Modal"]
// struct DisqualifyModal {
//     #[name = "Enter the user ID of the player you want to disqualify"]
//     #[placeholder = "Make sure the user ID is provided, not the username"]
//     user_id: u64,
// }

// pub async fn disqualify(
//     ctx: Context<'_>,
//     region: Region,
// ) -> Result<(), Error> {
//     let msg = ctx
//         .send(|s| {
//             s.ephemeral(true)
//                 .reply(true)
//                 .content("Attempting to disqualify player...")
//         })
//         .await?;
//     let user_id = {
//         let modal_result = create_modal::<DisqualifyModal>(&ctx, ctx.author().id).await?;
//         modal_result
//             .await_reply(ctx, |r| r.timeout(std::time::Duration::from_secs(60)))
//             .await?
//             .content
//             .parse::<u64>()
//             .map_err(|_| Error::Other("Invalid user ID provided".to_string()))?
//     };

//     let database = ctx.data().database.regional_databases.get(&region).unwrap();
//     let config = get_config(database).await;
//     let round_collection = get_round(&config);
//     let round = config.get("round").unwrap().as_i32().unwrap();
//     let collection = ctx
//         .data()
//         .database
//         .regional_databases
//         .get(&region)
//         .unwrap()
//         .collection::<Document>(round_collection.as_str());

//     let player = collection
//         .find_one(doc! {"discord_id": user_id.to_string()}, None)
//         .await?;

//     match player {
//         Some(player) => {
//             let match_id = player
//                 .get("match_id")
//                 .unwrap()
//                 .to_string()
//                 .parse::<i32>()
//                 .unwrap();
//             let mannequin = add_mannequin(&region, Some(match_id), None);
//             collection
//                 .delete_one(doc! {"discord_id": user_id.to_string()}, None)
//                 .await?;
//             collection.insert_one(mannequin, None).await?;
//             ctx.respond(format!("Successfully disqualified player: {}({}) with respective Discord <@{}> at round {}", 
//                     player.get("name").unwrap().as_str().unwrap(), 
//                     player.get("tag").unwrap().as_str().unwrap(),
//                     user_id,
//                     round))
//                 .await?;

//             Ok(())
//         }
//         None => {
//             ctx.respond(format!("No player is found for this ID at round {}", round))
//                 .await?;
//             Ok(())
//         }
//     }
// }
