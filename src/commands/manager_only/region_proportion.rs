use crate::{bracket_tournament::region::Region, Context, Error};
use mongodb::bson::{doc, Document};
use strum::IntoEnumIterator;
/// Get proportion of participants from each region
struct RegionStats {
    region: Region,
    count: i32,
    percentage: Option<f64>,
}

#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]

pub async fn region_proportion(ctx: Context<'_>) -> Result<(), Error> {
    let filter: Document = doc! { "name": { "$ne": "Mannequin" } }; //Filter out mannequins $ne = not equal
    let mut data: Vec<RegionStats> = vec![];
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let count: i32 = match database
            .collection::<i32>("Player")
            .count_documents(filter.clone(), None)
            .await
        {
            Ok(result) => result.try_into().unwrap_or_else(|_err| {
                // The _ to shut up warning lol
                let _ = ctx.say("There are way too many documents so it is unable to convert from u64 to i32 due to overflow.");
                0
            }),
            Err(err) => {
                ctx.say(format!("{}",err)).await?;
                0
            }
        };
        data.push(RegionStats {
            region,
            count,
            percentage: None,
        });
    }
    let total: i32 = data.iter().map(|x| x.count).sum();
    for region in data.iter_mut() {
        region.percentage = Some(region.count as f64 / total as f64 * 100.0);
    }

    ctx.send(|s| {
        s.reply(true).ephemeral(false).embed(|e| {
            e.title("Region Proportion")
                .description(
                    "The following statistics is collected from the registered participants.",
                )
                .fields(vec![
                    (
                        format!("{:?}", data[0].region),
                        format!(
                            "{} players \n{:.2}%",
                            data[0].count,
                            data[0].percentage.unwrap()
                        ),
                        true,
                    ),
                    (
                        format!("{:?}", data[1].region),
                        format!(
                            "{} players\n{:.2}%",
                            data[1].count,
                            data[1].percentage.unwrap()
                        ),
                        true,
                    ),
                    (
                        format!("{:?}", data[2].region),
                        format!(
                            "{} players\n{:.2}%",
                            data[2].count,
                            data[2].percentage.unwrap()
                        ),
                        true,
                    ),
                ])
        })
    })
    .await?;
    Ok(())
}
