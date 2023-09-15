use crate::{
    bracket_tournament::{config::get_config, region},
    database_utils::{
        find_discord_id::find_discord_id,
        find_enemy::{find_enemy, is_mannequin},
    },
    misc::{get_mode_icon, QuoteStripper},
    Context, Error,
};

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use tracing::{info, instrument};

/// View your opponent
#[instrument]
#[poise::command(slash_command, guild_only, rename = "view-opponent")]
pub async fn view_opponent(ctx: Context<'_>) -> Result<(), Error> {
    info!("Getting opponent for user {}", ctx.author().tag());
    let caller = match find_discord_id(&ctx, None, None).await {
        Some(caller) => caller,
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
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
    let round = config.get("round").unwrap().as_i32().unwrap();
    let current_round: Collection<Document> =
        database.collection(format!("Round {}", round).as_str());
    let caller: Document = match current_round
        .find_one(doc! {"tag": &caller_tag}, None)
        .await
    {
        Ok(Some(player)) => {
            if player.get("battle").unwrap().as_bool().unwrap() {
                ctx.send(|s| {
                    s.reply(true).ephemeral(true).embed(|e| {
                        e.title("You have already submitted the result!")
                            .description("Please wait until next round begins!")
                    })
                })
                .await?;
                return Ok(());
            } else {
                player
            }
        }
        Ok(None) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
                    e.title("You are not in this round!")
                        .description("Oops! Better luck next time")
                })
            })
            .await?;
            return Ok(());
        }
        Err(_) => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
                    e.title("An error pops up!")
                        .description("Please run this command later!")
                })
            })
            .await?;
            return Ok(());
        }
    };
    let enemy = match find_enemy(&ctx, &region, &round, &match_id, &caller_tag).await {
        Some(enemy) => {
            if is_mannequin(&enemy) {
                ctx.send(|s| {
                    s.reply(true).ephemeral(true).embed(|e| {
                        e.title("Congratulation! You are the bye player for this round!").description(
                            "Please run </submit-result:1148650981555441894> to be in next round!",
                        )
            })
        })
        .await?;
                return Ok(());
            } else {
                enemy
            }
        }
        None => {
            ctx.send(|s| {
                s.reply(true).ephemeral(true).embed(|e| {
                    e.title("An error pops up!")
                        .description("Please run this command later!")
                })
            })
            .await?;
            return Ok(());
        }
    };

    let enemy_tag = enemy.get("tag").unwrap().to_string().strip_quote();

    // let player1 = request(get_api_link("player", &caller_tag).as_str()).await?;
    // let player2 = request(get_api_link("player", &enemy_tag).as_str()).await?;

    ctx.send(|s| {
        s.reply(true).ephemeral(true).embed(|e| {
            e.title("**DISCORD BRAWL CUP TOURNAMENT**")
                .description(format!("Round {} - Match {}", round, match_id))
                .thumbnail(get_mode_icon(config.get("mode").unwrap().as_str().unwrap()))
                .fields(vec![
                    (caller.get("name").unwrap().to_string(), caller_tag, true),
                    ("".to_string(), "**VS**".to_string(), true),
                    (enemy.get("name").unwrap().to_string(), enemy_tag, true),
                ])
        })
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
