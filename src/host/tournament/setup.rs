use crate::bracket_tournament::bracket_update::update_bracket;
use crate::database::add::insert_mannequins;
use crate::database::config::get_config;
use crate::database::stat::count_registers;
use crate::database::update::{
    resetting_tournament_config, setting_tournament_config, update_round_1, update_round_config,
};
use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::bson::{doc, Bson::Null, Document};
use mongodb::Collection;
use poise::ReplyHandle;
use tracing::error;
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
        "<a:loading:1187839622680690689> Generating tournament bracket image...", //10
        "<:tick:1187839626338111600> Done! Tournament bracket image generated!", //11
        "<:sad:1187843167760949348> Tournament bracket image failed to generate!", //12
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
        Err("Not enough players to start the tournament!")?;
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
    update_round_1(ctx, region, rounds as i32).await?;
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
    // msg.edit(*ctx, |s| {
    //     s.embed(|e| {
    //         e.title("Setting up tournament").description(format!(
    //             "{}\n{}\n{}{}\n{}{}\n{}{}\n{}\n{}",
    //             prompts[0],
    //             prompts[1],
    //             prompts[3],
    //             &count,
    //             prompts[7],
    //             &rounds,
    //             prompts[5],
    //             &byes,
    //             prompts[9],
    //             prompts[10]
    //         ))
    //     })
    // })
    // .await?;
    // match update_bracket(ctx, Some(region)).await {
    //     Ok(_) => {
    //         msg.edit(*ctx, |s| {
    //             s.embed(|e| {
    //                 e.title("Setting up tournament").description(format!(
    //                     "{}\n{}\n{}{}\n{}{}\n{}{}\n{}\n{}",
    //                     prompts[0],
    //                     prompts[1],
    //                     prompts[3],
    //                     &count,
    //                     prompts[7],
    //                     &rounds,
    //                     prompts[5],
    //                     &byes,
    //                     prompts[9],
    //                     prompts[11]
    //                 ))
    //             })
    //         })
    //         .await?;
    //     }
    //     Err(e) => {
    //         msg.edit(*ctx, |s| {
    //             s.embed(|e| {
    //                 e.title("Setting up tournament").description(format!(
    //                     "{}\n{}\n{}{}\n{}{}\n{}{}\n{}\n{}",
    //                     prompts[0],
    //                     prompts[1],
    //                     prompts[3],
    //                     &count,
    //                     prompts[7],
    //                     &rounds,
    //                     prompts[5],
    //                     &byes,
    //                     prompts[9],
    //                     prompts[12]
    //                 ))
    //             })
    //         })
    //         .await?;
    //         return Err(e);
    //     }
    // }
    Ok(())
}

pub async fn starter_wrapper(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let config = get_config(ctx, region).await;
    match start_tournament(ctx, msg, region).await {
        Ok(_) => {
            prompt(
                ctx,
                msg,
                "Tournament starts!",
                format!("The tournament has begun for {}", region.full()),
                None,
                Some(0xFFFF0000),
            )
            .await?;
            Ok(())
        }
        Err(e) => {
            error!("{e}");
            revert(ctx, region).await?;
            resetting_tournament_config(ctx, region, Some(config)).await?;
            prompt(
                ctx,
                msg,
                "Failed to start tournament!",
                "<:sad:1187843167760949348> Failed to start tournament!",
                None,
                Some(0xFF0000),
            )
            .await?;
            Ok(())
        }
    }
}

async fn revert(ctx: &Context<'_>, region: &Region) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Players");
    collection
        .delete_many(doc! {"discord_id": Null}, None)
        .await?;
    let update = doc! {
        "$set": {
            "match_id": Null,
        }
    };
    collection.update_many(doc! {}, update, None).await?;
    let collections = database.list_collection_names(None).await?;
    for collection in collections {
        if collection.starts_with("Round") {
            database
                .collection::<Document>(&collection)
                .drop(None)
                .await?;
        }
    }
    Ok(())
}
