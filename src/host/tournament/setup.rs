use crate::bracket_tournament::bracket_update::update_bracket;
use crate::database::add::insert_mannequins;
use crate::database::stat::count_registers;
use crate::database::update::{resetting_tournament_config, setting_tournament_config, update_round_1, update_round_config};
use crate::{Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;
const MINIMUM_PLAYERS: i32 = 3; // The minimum amount of players required to start a tournament

pub async fn start_tournament(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let prompts = [
        "<:tick:1187839626338111600> Closed! Registration is now closed!", //0
        "<:tick:1187839626338111600> Opened! Tournament is opened!",       //1
        "<a:loading:1187839622680690689> Counting players...",             //2
        "<:tick:1187839626338111600> Counted! Players:  ",                 //3
        "<:sad:1187843167760949348> Not enough players to start the tournament! Aborting start!", //4
        "<:info:1187845402163167363> Byes: ", //5
        "<a:loading:1187839622680690689> Calculating rounds...", //6
        "<:tick:1187839626338111600> Calculated! Rounds: ", //7
        "<a:loading:1187839622680690689> Setting up first round", //8
        "<:tick:1187839626338111600> Done! First round is set!", //9
        "<:tick:1187839626338111600> Done! Tournament bracket image generated!", //10
    ];
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament")
                .description(format!("{}\n{}", prompts[0], prompts[1]))
        })
        .components(|c| c)
    })
    .await?;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament")
                .description(format!("{}\n{}\n{}", prompts[0], prompts[1], prompts[2]))
        })
    })
    .await?;
    setting_tournament_config(ctx, region).await?;
    let count = count_registers(ctx, region).await?;
    if count < MINIMUM_PLAYERS {
        msg.edit(*ctx, |s| {
            s.embed(|e| {
                e.title("Setting up tournament").description(format!(
                    "{}\n{}\n{}\n{}{}\n{}",
                    prompts[0], prompts[1], prompts[2], prompts[3], count, prompts[4]
                ))
            })
        })
        .await?;
        resetting_tournament_config(ctx, region).await?;
        return Ok(());
    }
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}",
                prompts[0], prompts[1], prompts[3], &count, prompts[6]
            ))
        })
    })
    .await?;
    let rounds = (count as f64).log2().ceil() as u32;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}{}",
                prompts[0], prompts[1], prompts[3], &count, prompts[7], &rounds
            ))
        })
    })
    .await?;
    let byes = 2_i32.pow(rounds) - count;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}{}\n{}{}",
                prompts[0], prompts[1], prompts[3], &count, prompts[7], &rounds, prompts[5], &byes
            ))
        })
    })
    .await?;
    update_round_config(ctx, region).await?;
    insert_mannequins(ctx, region, byes).await?;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}{}\n{}{}\n{}",
                prompts[0],
                prompts[1],
                prompts[3],
                &count,
                prompts[7],
                &rounds,
                prompts[5],
                &byes,
                prompts[8]
            ))
        })
    })
    .await?;
    update_round_1(ctx, region).await?;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}{}\n{}{}\n{}",
                prompts[0],
                prompts[1],
                prompts[3],
                &count,
                prompts[7],
                &rounds,
                prompts[5],
                &byes,
                prompts[9]
            ))
        })
    })
    .await?;
    update_bracket(ctx, Some(region)).await?;
    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Setting up tournament").description(format!(
                "{}\n{}\n{}{}\n{}{}\n{}{}\n{}\n{}",
                prompts[0],
                prompts[1],
                prompts[3],
                &count,
                prompts[7],
                &rounds,
                prompts[5],
                &byes,
                prompts[9],
                prompts[10]
            ))
        })
    })
    .await?;

    Ok(())
}
