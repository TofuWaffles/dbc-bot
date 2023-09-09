use crate::{misc::CustomError, self_role::SelfRoleMessage, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]

pub async fn create_self_role_message(
    ctx: Context<'_>,
    #[description = "The role ID to assign"] role_id: String,
    #[description = "The content of the self-roles message"] content: String,
    #[description = "The channel ID to send the self-roles message"] channel_id: String,
    #[description = "The channel ID to send the self-roles message"] ping_channel_id: String,
    #[description = "The custom emoji for the react button"] emoji: String,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(guild_id) => guild_id,
        None => {
            ctx.send(|s|
                s.content("Discord is so trash bro. They cannot even provide the server ID lol")
                .ephemeral(true)
                .reply(true)
            )
                .await?;
            return Ok(());
        }
    };

    let emoji_obj = serenity::parse_emoji(&emoji)
        .ok_or_else(|| CustomError("Invalid emoji provided.".to_owned()))?;

    let message = serenity::ChannelId(channel_id.parse::<u64>().unwrap())
        .send_message(&ctx, |m| {
            m.content(content).components(|c| {
                c.create_action_row(|a| {
                    a.create_button(|b| {
                        b.style(serenity::ButtonStyle::Success)
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
        .database
        .general
        .collection::<SelfRoleMessage>("SelfRoleMessage");

    self_role_messages
        .insert_one(self_role_message, None)
        .await?;

    ctx.say(format!(
        "Custom self role message created in <#{}>",
        channel_id
    ))
    .await?;

    Ok(())
}
