use super::register::{player_registered, register_opened};
use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::region::Region;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude::{self as serenity};
use tracing::{info, instrument};

/// Remove your registration from Discord Brawl Cup.
#[instrument]
#[poise::command(slash_command, guild_only)]
pub async fn deregister(ctx: Context<'_>) -> Result<(), Error> {
    info!("Attempted to deregister user {}", ctx.author().tag());
    let player = match player_registered(&ctx, None).await? {
        None => {
            ctx.send(|s|{
                s.reply(true)
                .ephemeral(true)
                .embed(|e|{
                    e.title("**You have not registered!**")
                    .description("You have not registered for the tournament! If you want to register, please use the </register:1145363516325376031> command!")
                })
            }).await?;
            return Ok(());
        }
        Some(data) => data,
    };

    let region = Region::find_key(player.get("region").unwrap().as_str().unwrap()).unwrap();
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;

    if !register_opened(&ctx, &config).await? {
        return Ok(());
    }

    let deregister_confirm: u64 = format!("{}1", ctx.id()).parse().unwrap();
    let deregister_cancel: u64 = format!("{}0", ctx.id()).parse().unwrap();
    ctx.send(|s|{
        s.components(|c|{
          c.create_action_row(|a|{
            a.create_button(|b|{
              b.label("Confirm")
              .style(serenity::ButtonStyle::Success)
              .custom_id(deregister_confirm)
            })
            .create_button(|b|{
              b.label("Cancel")
              .style(serenity::ButtonStyle::Danger)
              .custom_id(deregister_cancel)
            })
          })
        })
          .reply(true)
          .ephemeral(true)
          .embed(|e|{
            e.title("**Are you sure you want to deregister?**")
            .description(format!("You are about to deregister from the tournament. Below information are what you told us!\nYour account name: **{}**\nWith your respective tag: **{}**\nAnd you are in the following region: **{}**", 
                                player.get("name").unwrap().to_string().strip_quote(), 
                                player.get("tag").unwrap().to_string().strip_quote(), 
                                Region::find_key(player.get("region").unwrap().to_string().strip_quote().as_str()).unwrap()) 
                        )
        })
    }).await?;

    if let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .await
    {
        if mci.data.custom_id == deregister_confirm.to_string() {
            let region = Region::find_key(&player.get("region").unwrap().to_string().strip_quote());
            let player_data: Collection<Document> = ctx
                .data()
                .database
                .regional_databases
                .get(&region.unwrap())
                .unwrap()
                .collection("Players");
            player_data
                .delete_one(doc! {"_id": player.get("_id")}, None)
                .await?;

            let mut confirm_prompt = mci.message.clone();
            confirm_prompt.edit(ctx,|s| {
                s.components(|c| {c})
                    .embed(|e| {
                        e.title("**Deregistration is successful**")
                            .description(
                            "Seriously, are you leaving us? We hope to see you in the next tournament!",
                        )
                  })
            })
            .await?;
        } else if mci.data.custom_id == deregister_cancel.to_string() {
            let mut cancel_prompt = mci.message.clone();
            cancel_prompt
                .edit(ctx, |s| {
                    s.components(|c| c).embed(|e| {
                        e.title("**Deregistration cancelled!**")
                            .description("Thanks for staying in the tournament with us!")
                    })
                })
                .await?;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
        mci.message.delete(ctx).await?;
    }
    Ok(())
}
