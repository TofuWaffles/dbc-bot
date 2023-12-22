use crate::{Context, Error, database::stat::count_registers};
use poise::ReplyHandle;
use dbc_bot::Region;
const MINIMUM_PLAYERS: i32 = 3; // The minimum amount of players required to start a tournament

pub async fn start_tournament(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region
) -> Result<(), Error> {
    let prompts = vec![
        "<a:loading:1187839622680690689> Counting players...", //0
        "<:tick:1187839626338111600> Counted! Players:  ", //1
        "<:sad:1187843167760949348> Not enough players to start the tournament! Aborting start!", //2
        "<:info:1187845402163167363> Byes: ", //3
        "<a:loading:1187839622680690689> Calculating rounds...", //4
        "<:tick:1187839626338111600> Calculated! Rounds: ", //5
        ""
    ];
    msg.edit(*ctx, |s| 
      s.embed(|e| {
        e.title("Setting up tournament")
        .description(prompts[0])
      })
    ).await?;
    let count = count_registers(ctx, region).await?;
    if count < MINIMUM_PLAYERS {
        msg.edit(*ctx, |s| 
          s.embed(|e| {
            e.title("Setting up tournament")
            .description(format!("{}{}\n{}",prompts[1],count,prompts[2]))
          })
        ).await?;
        return Ok(());
    }
    let rounds = (count as f64).log2().ceil() as u32;
    let byes = 2_i32.pow(rounds) - count;
    msg.edit(*ctx, |s| 
      s.embed(|e| {
        e.title("Setting up tournament")
        .description(format!("{}{}\n{}{}", prompts[1], &count, prompts[3], &byes))
      })
    ).await?;
    
    Ok(())
}





