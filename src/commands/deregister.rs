use crate::bracket_tournament::config::get_config;
use crate::bracket_tournament::region::Region;
use crate::database_utils::find_discord_id::find_discord_id;
use crate::misc::QuoteStripper;
use crate::{Context, Error};
use mongodb::{
    bson::{doc, Document},
    Collection,
};
use poise::serenity_prelude::{self as serenity};

/// Remove your registration from Discord Brawl Cup.
#[poise::command(slash_command, guild_only)]
pub async fn deregister(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let data = match find_discord_id(&ctx, None).await {
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
    let region = Region::find_key(data.get("region").unwrap().as_str().unwrap()).unwrap();
    //Check whether registation has already closed
    let database = ctx.data().database.regional_databases.get(&region).unwrap();
    let config = get_config(database).await;

    if !(config.get("registration").unwrap()).as_bool().unwrap() {
        ctx.send(|s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("**Registration has already closed!**")
                    .description("Sorry, registration has already closed!")
            })
        })
        .await?;
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
            .description(format!("You are about to deregister from the tournament. Below information are what you told us!\n
                                Your account name: {} \n
                                With your respective tag: {}\n
                                And you are in the following region: {}", 
                                data.get("name").unwrap().to_string().strip_quote(), data.get("tag").unwrap().to_string().strip_quote(), format!("{:?}", Region::find_key(data.get("region").unwrap().to_string().strip_quote().as_str()))) 
                        )
        })
    }).await?;

    while let Some(mci) = serenity::CollectComponentInteraction::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .await
    {
        if mci.data.custom_id == deregister_confirm.to_string() {
            let region = Region::find_key(&data.get("region").unwrap().to_string().strip_quote());
            let player_data: Collection<Document> = ctx
                .data()
                .database
                .regional_databases
                .get(&region.unwrap())
                .unwrap()
                .collection("Player");
            player_data
                .delete_one(doc! {"_id": data.get("_id")}, None)
                .await?;

            let mut confirm_prompt = mci.message.clone();
            confirm_prompt.edit(ctx,|s| {
                s.components(|c| {c})
                    .embed(|e| {
                        e.title("**Deregistration is successful**").description(
                            "Seriously, are you leaving us? We hope to see you in the next tournament!",
                        )
                  })
            })
            .await?;
            break;
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
            break;
        }
    }
    Ok(())
}
