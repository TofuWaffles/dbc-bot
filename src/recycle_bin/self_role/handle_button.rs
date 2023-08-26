use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity, Context, RoleId};

pub async fn handle_selfrole_button(
    button_interaction: &serenity::MessageComponentInteraction,
    ctx: &Context,
    data: &Data,
) -> Result<(), Error> {
    if button_interaction.data.custom_id != "test" {
        return Ok(());
    }

    let self_role_message = match data
        .self_role_messages
        .get(&(button_interaction.message.id.0 as i64))
    {
        Some(self_role_message) => self_role_message,
        None => return Ok(()),
    };

    todo!();
}

// pub async fn handle_self_role_react(
//     ctx: &Context,
//     data: &Data,
// ) -> Result<(), Error> {
//     // TODO: Change custom ID
//     if button_interaction.data.custom_id != "test" {
//         return Ok(());
//     }

    let self_role_message = match data
        .self_role_messages
        .get(&(button_interaction.message.id.0 as i64))
    {
        Some(self_role_message) => self_role_message,
        None => return Ok(()),
    };

    todo!();
}