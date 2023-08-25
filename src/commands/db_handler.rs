use mongodb::bson::doc;
use crate::bracket_tournament::player::PlayerDB;
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn get_player_data(ctx: Context<'_>, #[description = "Check a player registration status by username here" ] username: String) -> Result<(), Error> {
    let player_data = ctx.data().client.database("DBC-bot").collection("PlayerDB");
    let individual_player: PlayerDB = player_data.find_one(doc! {
        "username": &username
    }, None).await?.expect(&format!("Missing: {} document.", &username));

    ctx.channel_id().send_message(&ctx, |response|{
        response
        .allowed_mentions(|a| a.replied_user(true))
        .embed(|e|{
           e.title(format!("**{}**", individual_player.username))
            .description(individual_player.tag.to_string())
        })
    }).await?;
    Ok(())
}
