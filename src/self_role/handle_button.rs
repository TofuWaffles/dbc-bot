use crate::{Data, Error};
use poise::serenity_prelude::{self as serenity, Context};

pub async fn handle_selfrole_button(
    button_interaction: &serenity::MessageComponentInteraction,
    ctx: &Context,
    data: &Data,
) -> Result<(), Error> {
    // TODO: Change custom ID
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

    let role_id = self_role_message.role_id as u64;

    button_interaction
        .to_owned()
        .member
        .unwrap()
        .add_role(ctx, role_id)
        .await?;

    Ok(())
}

// pub async fn handle_self_role_react(
//     ctx: &Context,
//     reaction: &Reaction,
//     data: &Data,
// ) -> Result<(), Error> {
//     if !reaction.emoji.unicode_eq("✅")
//         || !data
//             .self_role_messages
//             .contains_key(&(reaction.message_id.0 as i64))
//     {
//         return Ok(());
//     }

//     let self_role_message = data
//         .self_role_messages
//         .get(&(reaction.message_id.0 as i64))
//         .unwrap();

//     let mut member = reaction
//         .guild_id
//         .unwrap()
//         .member(ctx, reaction.user_id.unwrap())
//         .await?;

//     if !member
//         .roles
//         .contains(&RoleId::from(self_role_message.role_id as u64))
//     {
//         member
//             .add_role(ctx, RoleId::from(self_role_message.role_id as u64))
//             .await?;
//     }

//     // if self_role_message.ping_channel_id != 0 {
//     //     let target_ping_channel = reaction
//     //         .guild_id
//     //         .unwrap()
//     //         .channels(ctx)
//     //         .await?
//     //         .get(&(serenity::ChannelId::from(self_role_message.ping_channel_id as u64)))
//     //         .unwrap()
//     //         .say(ctx, format!("Welcome, <@{}>!", reaction.user_id.unwrap().0));
//     // }

//     Ok(())
// }

// pub async fn handle_self_role_unreact(
//     ctx: &Context,
//     reaction: &Reaction,
//     data: &Data,
// ) -> Result<(), Error> {
//     if !reaction.emoji.unicode_eq("✅")
//         || !data
//             .self_role_messages
//             .contains_key(&(reaction.message_id.0 as i64))
//     {
//         return Ok(());
//     }

//     let self_role_message = data
//         .self_role_messages
//         .get(&(reaction.message_id.0 as i64))
//         .unwrap();

//     let mut member = reaction
//         .guild_id
//         .unwrap()
//         .member(ctx, reaction.user_id.unwrap())
//         .await?;

//     if member
//         .roles
//         .contains(&RoleId::from(self_role_message.role_id as u64))
//     {
//         member
//             .remove_role(ctx, RoleId::from(self_role_message.role_id as u64))
//             .await?;
//     }

//     Ok(())
// }
