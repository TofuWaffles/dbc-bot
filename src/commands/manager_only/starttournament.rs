use crate::bracket_tournament::config::disable_registration;
use crate::bracket_tournament::{region::Region, *};
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    options::AggregateOptions,
    Collection,
};
use strum::IntoEnumIterator;

///Run this command once registration closes to start the tournament.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn start_tournament(ctx: Context<'_>) -> Result<(), Error> {
    //Handling each region mathematical computations to preset brackets
    ctx.say("Starting tournament...").await?;
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();

        //Disable registration
        let config = database
            .collection::<Document>("Config")
            .find_one(None, None)
            .await
            .unwrap()
            .unwrap();
        database
            .collection::<Document>("Config")
            .update_one(config, disable_registration(), None)
            .await
            .unwrap();

        //Counting players in a region
        let collection: Collection<Document> = database.collection("Player");
        let count: i32 = collection
            .count_documents(None, None)
            .await
            .unwrap()
            .try_into()
            .unwrap();

        //If there is no players in a region, skip to next region
        if count < 3 {
            ctx.say(
                format!(
                    "Aborting organizing tournament for {} due to lacking of players",
                    region
                )
                .as_str(),
            )
            .await?;
            continue;
        }
        let rounds = (count as f64).log2().ceil() as u32;
        let byes = 2_i32.pow(rounds) - count;
        ctx.channel_id()
            .send_message(ctx, |m| {
                m.content(format!("There are {} byes in region {}", byes, region))
            })
            .await?;
        match byes {
            0 => {}
            _ => {
                for bye in 1..=byes {
                    let mannequin = mannequin::add_mannequin(&region, Some(bye), None);
                    collection.insert_one(mannequin, None).await?;
                }
            }
        }
        assign_match_id::assign_match_id(&region, &database, byes).await?;
        //Create rounds collection for each databases
        for round in 1..=rounds {
            let collection_names = format!("Round {}", round);
            if !database
                .list_collection_names(None)
                .await
                .unwrap()
                .contains(&collection_names)
            {
                database
                    .create_collection(format!("Round {}", round), None)
                    .await?;
            }
        }

        //Clone and sort all player data to round 1
        let pipeline = vec![
            doc! {
                "$sort": {
                    "match_id": 1
                }
            },
            doc! {
                "$out": "Round 1"
            },
        ];
        let aggregation_options = AggregateOptions::builder().allow_disk_use(true).build();

        // Run the aggregation pipeline to copy and sort documents
        collection
            .aggregate(pipeline, Some(aggregation_options))
            .await?;
    }
    ctx.channel_id()
        .send_message(ctx, |m| m.content("Setting up done!"))
        .await?;
    Ok(())
}
