// use crate::bracket_tournament::battle_log::BattleLog;
// use crate::utils::mode::get_mode_icon;
// use crate::{Context, Error};
// use poise::serenity_prelude::Colour;

// /// Get the player's profile
// #[poise::command(slash_command, prefix_command)]
// pub async fn latest_log(
//   ctx: Context<'_>, 
//   #[description = "Put your tag here (without #)" ] tag: String) 
// -> Result<(), Error>{
//   let log = BattleLog::new(&tag).await;
//   let latest_log = log.get_latest_log();
//   ctx.say(format!(
//     "Time: {}
//     Mode: {}
//      Map: {}",&latest_log.battle_time,&latest_log.event.get(mode), &latest_log.event.map)).await?;

//   // ctx.channel_id()
//   //   .send_message(&ctx, |response|{
//   //     response
//   //       .allowed_mentions(|a| a.replied_user(true))
//   //       .embed(|e|{
//   //          e.title(String::from("Latest Battle Log"))
//   //           .thumbnail(format!("{:#?}", get_mode_icon().get(&latest_log.event.mode)))
//   //           .color(color(&latest_log.battle.result))
//   //           .fields(vec![
//   //             ("Mode: ",&latest_log.event.mode, true),
//   //             ("Map: ", &latest_log.event.map, true),
//   //             ("Type: ", &latest_log.battle.battle_type, true),
//   //             ("Result: ", &latest_log.battle.result, true),
//   //             ("Duration: ", &latest_log.battle.duration.to_string(), true),
//   //             ("Trophy Change: ", &latest_log.battle.trophy_change.to_string(), true),
//   //             ("Star Player: ", &latest_log.battle.star_player.name, true),
//   //           ])
//   //         .timestamp(ctx.created_at())
//   //       })
//   //   }).await?;
//   Ok(())
// }

// fn color(result: &str) -> Colour {
//   match result {
//       "victory" => Colour::new(u32::from_str_radix("00800",16).unwrap()), // Green
//       "defeat" => Colour::new(u32::from_str_radix("FF0000",16).unwrap()), // Red
//       "draw" => Colour::new(u32::from_str_radix("FFFFFF",16).unwrap()), // White
//       _ => Colour::new(000000),         // Default color (black) for unknown cases
//   }
// }