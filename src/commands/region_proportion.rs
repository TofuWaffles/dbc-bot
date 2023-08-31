// use crate::{Context, Error};
// use futures::StreamExt;
// use mongodb::{
//     bson::{doc, Document},
//     options::AggregateOptions,
//     Collection,
// };
// use std::collections::HashMap;
// /// Get proportion of participants from each region
// #[poise::command(slash_command)]
// pub async fn region_proportion(ctx: Context<'_>) -> Result<(), Error> {
//     let collection: Collection<Document> = ctx.data().database.collection("Player");
//     let pipeline = vec![
//         doc! {
//             "$group": {
//                 "_id": "$region",
//                 "count": { "$sum": 1 }
//             }
//         },
//         doc! {
//             "$project": {
//                 "_id": 0,
//                 "region": "$_id",
//                 "count": 1
//             }
//         },
//     ];

//     let mut cursor = collection
//         .aggregate(
//             pipeline,
//             AggregateOptions::builder().allow_disk_use(true).build(),
//         )
//         .await?;

//     let mut total_count = 0;
//     let mut region_counts: HashMap<String, i32> = HashMap::new();
//     while let Some(Ok(document)) = cursor.next().await {
//         let region = document.get_str("region").unwrap();
//         let count = document.get_i32("count").unwrap();
//         total_count += count;
//         region_counts.insert(region.to_string(), count);
//     }

//     let mut data: Vec<Vec<RegionStats>> = vec![];
//     let live = region_counts.clone();
//     for (region, count) in live{
//         let proportion = (count as f64 / total_count as f64) * 100.0;
//         data.push(vec![RegionStats::Region(&region), RegionStats::Count(count), RegionStats::Proportion(proportion)]);
//     }
//     println!("{:?}", region_counts);    
//     ctx.send(|s|{
//       s.reply(true)
//         .ephemeral(false)
//         .embed(|e|{
//           e.title("Region Proportion")
//             .description("The following statistics is collected from the registered participants.")
//             .fields(vec![("Region",format!("{:?}\n{:?}\n{:?}",data[0][0],data[1][0], data[2][0]),true),
//                         ("Count",format!("{:?}\n{:?}\n{:?}",data[0][1],data[1][1], data[2][1]),true),
//                         ("Proportion",format!("{:?}\n{:?}\n{:?}",data[0][1],data[1][1], data[2][1]),true)
//                         ])
//         })
//     }).await?;
//     Ok(())
// }

// #[derive(Debug)]
// enum RegionStats {
//   Region(&'static str),
//   Count(i32),
//   Proportion(f64),
// }