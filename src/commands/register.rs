use crate::bracket_tournament::player::{Player};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn register(ctx: Context<'_>, #[description = "Put your tag here (without #)" ] tag: String) -> Result<(), Error>{
    let player_register_data = Player::new(&tag).await;
    let player_data = ctx.data().client.database("DBC-bot").collection("PlayerDB");
    
    player_data.insert_one(player_register_data, None).await?;
    ctx.channel_id().send_message(&ctx, |response|{
        response
        .allowed_mentions(|a| a.replied_user(true))
        .embed(|e|{
           e.title(String::from("You've been registered for the tournament!"))
        })
    }).await?;
    Ok(())
}