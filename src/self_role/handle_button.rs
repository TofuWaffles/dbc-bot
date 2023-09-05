use crate::{
    Data,
    Error
};
use poise::serenity_prelude::{self as serenity, Context};
use tokio::time::{sleep, Duration};

pub async fn handle_selfrole_button(
    button_interaction: &serenity::MessageComponentInteraction,
    ctx: &Context,
    data: &Data,
) -> Result<(), Error> {
    if button_interaction.data.custom_id != "register" {
        return Ok(());
    }

    let self_role_message = match data
        .self_role_messages
        .get(&(button_interaction.message.id.0 as i64))
    {
        Some(self_role_message) => self_role_message,
        None => return Ok(()),
    };

    let role_id = self_role_message.role_id as u64;

    let ping_channel_id = self_role_message.ping_channel_id as u64;

    let mut member = button_interaction.member.as_ref().unwrap().clone();
    let roles = member.guild_id.roles(ctx).await.unwrap();

    if let Some(role) = roles.get(&poise::serenity_prelude::RoleId(role_id)) {
        if let Err(err) = member.add_role(&ctx, role.id).await {
            eprintln!("Error adding role to member: {:?}", err);
            return Ok(());
        }
    }

    let ping_channel = match ctx.cache.guild_channel(ping_channel_id) {
        Some(ping_channel) => ping_channel,
        None => {
            eprintln!("Error retrieving ping channel");
            return Ok(());
        }
    };

    let sent_message = match ping_channel
        .send_message(&ctx, |m| {
            m.content(format!("<@{}>", member.user.id)).embed(|e| {
                e.description("You have been registered for the tournament!")
                    .color(poise::serenity_prelude::Colour::DARK_GREEN)
            })
        })
        .await
    {
        Ok(sent_message) => sent_message,
        Err(err) => {
            eprintln!("Error sending ping message: {:?}", err);
            return Ok(());
        }
    };

    button_interaction.create_interaction_response(ctx, |c| c.interaction_response_data(|d| d.embed(|e|e.description(format!("Registration was successful, please check the <#{}> channel for more information.", ping_channel_id))).ephemeral(true))).await?;

    sleep(Duration::from_secs(15)).await;

    if let Err(err) = sent_message.delete(&ctx).await {
        eprintln!("Error deleting ping message: {:?}", err);
        return Ok(());
    }

    Ok(())
}
