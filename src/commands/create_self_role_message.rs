use crate::{Context, Error};
use crate::self_role::SelfRoleMessage;
use crate::bracket_tournament::api::api_handlers::CustomError;

#[poise::command(slash_command, prefix_command)]
pub async fn create_self_role_message(
    ctx: Context<'_>,
    #[description = "The role ID to assign"] role_id: String,
    #[description = "The content of the self-roles message"] content: String,
    #[description = "The channel ID to send the self-roles message"] channel_id: String,
    #[description = "The channel ID to send the self-roles message"] ping_channel_id: String,
    #[description = "The custom emoji for the react button"] emoji: String,
) -> Result<(), Error> {

    let guild_id = ctx.guild_id().unwrap();

    let emoji_obj = poise::serenity_prelude::parse_emoji(&emoji).ok_or_else(|| CustomError("Invalid emoji provided.".to_owned()))?;

    let message = poise::serenity_prelude::ChannelId(channel_id.parse::<u64>().unwrap())
        .send_message(&ctx, |m| {
            m.content(content)
                .components(|c| {
                    c.create_action_row(|a| {
                        a.create_button(|b| {
                            b.style(poise::serenity_prelude::ButtonStyle::Success)
                                .custom_id("register")
                                .label("Register")
                                .emoji(emoji_obj)
                        })
                    })
                })
        })
        .await?;

    let self_role_message = SelfRoleMessage {
        message_id: message.id.0 as i64,
        guild_id: guild_id.0 as i64,
        role_id: role_id.parse::<i64>().unwrap(),
        ping_channel_id: ping_channel_id.parse::<i64>().unwrap(),
    };

    let self_role_messages = ctx
        .data()
        .db_client
        .database("DBC-bot")
        .collection::<SelfRoleMessage>("SelfRoles");

    self_role_messages.insert_one(self_role_message, None).await?;

    ctx.say(format!("Custom self role message created in <#{}>", channel_id)).await?;

    Ok(())
}
