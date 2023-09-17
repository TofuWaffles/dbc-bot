use std::io::Cursor;

use crate::{
    bracket_tournament::{config::get_config, region},
    database_utils::{
        battle_happened::battle_happened,
        find_discord_id::find_discord_id,
        find_enemy::{find_enemy, is_mannequin},
        find_round::get_round,
    },
    misc::QuoteStripper,
    visual::pre_battle::generate_pre_battle_img,
    Context, Error,
};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude as serenity;
use tracing::{info, instrument};

/// View your opponent
#[instrument]
#[poise::command(slash_command, guild_only, rename = "view-opponent")]
pub async fn view_opponent(ctx: Context<'_>) -> Result<(), Error> {
    info!("Getting opponent for user {}", ctx.author().tag());
    let msg = ctx
        .send(|s| {
            s.ephemeral(true)
                .reply(true)
                .content("Getting your opponent...")
        })
        .await?;
    let caller = match find_discord_id(&ctx, None, None).await {
        Some(caller) => caller,
        None => {
            msg.edit(ctx, |s| {
                s.embed(|e| {
                    e.title("You are not in the tournament!")
                        .description("Sorry, you are not in the tournament to use this command!")
                })
            })
            .await?;
            return Ok(());
        }
    };

    //Get player document via their discord_id
    let match_id: i32 = (caller.get("match_id").unwrap()).as_i32().unwrap();
    let caller_tag = caller.get("tag").unwrap().to_string().strip_quote();
    let region = region::Region::find_key(
        caller
            .get("region")
            .unwrap()
            .to_string()
            .strip_quote()
            .as_str(),
    )
    .unwrap();

    //Check if the user has already submitted the result or not yet disqualified
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;
    let current_round: Collection<Document> = database.collection(get_round(&config).as_str());
    let round = config.get("round").unwrap().as_i32().unwrap();
    let caller = match battle_happened(&ctx, &caller_tag, current_round, &msg).await? {
        Some(caller) => caller, // Battle did not happen yet
        None => return Ok(()),  // Battle already happened
    };
    let enemy = match find_enemy(&ctx, &region, &round, &match_id, &caller_tag).await {
        Some(enemy) => {
            if is_mannequin(&enemy) {
                msg.edit(ctx, |s|
                    s.embed(|e| {
                            e.title("Congratulations! You are the bye player for this round!")
                                .description("Please run </submit-result:1148650981555441894> to be in the next round!")
                        })
                )
                .await?;
                return Ok(());
            } else {
                enemy
            }
        }
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
                    e.title("An error occurred!")
                        .description("Please run this command later.")
                })
            })
            .await?;
            return Ok(());
        }
    };

    let image = generate_pre_battle_img(caller, enemy, &config)
        .await
        .unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    image.write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)?;
    let attachment = serenity::model::channel::AttachmentType::Bytes {
        data: bytes.into(),
        filename: "pre_battle.png".to_string(),
    };
    ctx.send(|s| {
        s.reply(true)
            .ephemeral(true)
            .embed(|e| {
                e.title("**DISCORD BRAWL CUP TOURNAMENT**")
                    .description(format!("Round {} - Match {}", round, match_id))
            })
            .attachment(attachment)
    })
    .await?;
    Ok(())
}

///View list of roles as manager of the tournament
#[instrument]
#[poise::command(slash_command, guild_only, rename = "view-managers")]
pub async fn view_managers(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let database = &ctx.data().database.general;
    let mut list: Vec<String> = vec![];
    let mut managers = database
        .collection::<Document>("Managers")
        .find(doc! {"guild_id": &guild_id}, None)
        .await?;
    while let Some(manager) = managers.try_next().await? {
        let role_id = manager.get("role_id").unwrap().to_string().strip_quote();
        list.push(role_id);
    }
    let role_msg = list
        .iter()
        .map(|role| format!("<@&{}>", role))
        .collect::<Vec<String>>()
        .join(", ");
    ctx.send(|s| {
        s.reply(true).ephemeral(true).embed(|e| {
            e.title("**These following roles have permission to run manager-only commands: **")
                .description(role_msg)
        })
    })
    .await?;
    Ok(())
}
