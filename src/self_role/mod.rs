// pub mod handle_button;

// use crate::{Context, Error};
// use poise::serenity_prelude::{self as serenity, ButtonStyle, CreateButton, MessageBuilder};

// // The context will hold this type in a dash map indexed by its message_id for convenient searching
// #[derive(Debug, Clone, Copy)]
// pub struct SelfRoleMessage {
//     pub message_id: i64,
//     pub guild_id: i64,
//     pub role_id: i64,
//     pub ping_channel_id: i64,
// }

// /// Set up a self-role message in the current room.
// ///
// /// You can also optionally set a channel to ping the users who received the role.
// /// The ping message will be brief and will disappear within a second.
// #[poise::command(slash_command, guild_only)]
// pub async fn selfrole(
//     ctx: Context<'_>,
//     #[description = "Optional channel for the alert pings to be sent to (enter channel ID)"]
//     ping_channel_id: Option<serenity::ChannelId>,
//     #[description = "The role to give to users and to ping in the new channel (if a channel id is provided)"]
//     role: serenity::Role,
//     #[description = "The message to send for the self-role message"] message: String,
// ) -> Result<(), Error> {
//     let guild_id = match ctx.guild_id() {
//         Some(guild_id) => guild_id,
//         None => {
//             ctx.say("This command can only be used in a server.")
//                 .await?;
//             return Ok(());
//         }
//     };

//     let button = CreateButton::default()
//         .label("test")
//         .style(ButtonStyle::Primary)
//         .custom_id("test")
//         .to_owned();

//     let sent_message = ctx
//         .channel_id()
//         .send_message(&ctx, |m| {
//             m.content(message)
//                 .tts(true)
//                 .components(|c| c.create_action_row(|ar| ar.add_button(button)))
//         })
//         .await?;

//     let self_role_message = SelfRoleMessage {
//         message_id: sent_message.id.0 as i64,
//         guild_id: guild_id.0 as i64,
//         role_id: role.id.0 as i64,
//         /* If there is no channel id, we want to set it to 0
//         so that the event handler knows that we don't want to alert users who get the self role */
//         ping_channel_id: ping_channel_id.unwrap_or_else(|| serenity::ChannelId(0)).0 as i64,
//     };

//     // sqlx::query!(
//     //     "
//     //     INSERT INTO self_role_message(message_id, guild_id, role_id, ping_channel_id)
//     //     VALUES ($1, $2, $3, $4);
//     //     ",
//     //     self_role_message.message_id,
//     //     self_role_message.guild_id,
//     //     self_role_message.role_id,
//     //     self_role_message.ping_channel_id
//     // )
//     // .execute(&ctx.data().db_pool)
//     // .await?;

//     ctx.data()
//         .self_role_messages
//         .insert(self_role_message.message_id, self_role_message);

//     Ok(())
// }
