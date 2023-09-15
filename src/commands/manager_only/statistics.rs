use crate::{bracket_tournament::region::Region, checks::user_is_manager, Context, Error};
use mongodb::bson::doc;
use strum::IntoEnumIterator;
/// Get proportion of participants from each region
struct RegionStats {
    region: Region,
    count: i32,
    percentage: Option<f64>,
}

#[poise::command(slash_command, guild_only)]
pub async fn region_proportion(ctx: Context<'_>) -> Result<(), Error> {
    if !user_is_manager(ctx).await? {
        return Ok(());
    }

    let mut data: Vec<RegionStats> = get_region_stats(&ctx).await?;
    calculate_percentages(&mut data);
    let fields = data
        .iter()
        .map(|region| {
            (
                format!("{:?}", region.region),
                format!(
                    "{} players\n{:.2}%",
                    region.count,
                    region.percentage.unwrap()
                ),
                true,
            )
        })
        .collect::<Vec<_>>();

    ctx.send(|s| {
        s.reply(true).ephemeral(false).embed(|e| {
            e.title("Region Proportion")
                .description(
                    "The following statistics are collected from the registered participants.",
                )
                .fields(fields)
        })
    })
    .await?;

    Ok(())
}

async fn get_region_stats(ctx: &Context<'_>) -> Result<Vec<RegionStats>, Error> {
    let mut data: Vec<RegionStats> = vec![];

    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        let count: i32 = match database
            .collection::<i32>("Player")
            .count_documents(doc! { "name": { "$ne": "Mannequin" } }, None)
            .await
        {
            Ok(result) => result.try_into().unwrap_or_else(|_err| {
                let drop = ctx.say("There are way too many documents, so it is unable to convert from u64 to i32 due to overflow.");
                std::mem::drop(drop);
                0
            }),
            Err(err) => {
                ctx.say(format!("{}", err)).await?;
                0
            }
        };

        data.push(RegionStats {
            region,
            count,
            percentage: None,
        });
    }

    Ok(data)
}

fn calculate_percentages(data: &mut Vec<RegionStats>) {
    let total: i32 = data.iter().map(|x| x.count).sum();
    for region in data.iter_mut() {
        region.percentage = Some(region.count as f64 / total as f64 * 100.0);
    }
}
