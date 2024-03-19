use crate::{
    discord::{checks::is_host, prompt::prompt, role::get_roles_from_user},
    Context, Error,
};
use poise::serenity_prelude as serenity;
use tracing::error;

/// Displays your or another user's account creation date
#[poise::command(slash_command, guild_only, check = "is_host")]
pub async fn info(
    ctx: Context<'_>,
    #[description = "User id: "] user: String,
) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("Getting user info...")
                    .description("Hold on a sec...")
            })
        })
        .await?;
    let u = match serenity::UserId(user.parse::<u64>()?).to_user(&ctx).await {
        Ok(u) => u,
        Err(e) => {
            error!("{e}");
            return prompt(
                &ctx,
                &msg,
                "Unable to find the user. Please make sure you are using the correct id.",
                "Usage: `/info @id`",
                None,
                None,
            )
            .await;
        }
    };
    let user_info = format!(
        r#"
Name: **{name}**,
Id: `{id}`,
Created at: <t:{timestamp}:R> (<t:{timestamp}:F>)
Mention: <@{id}>"#,
        name = u.name,
        id = u.id,
        timestamp = u.created_at().unix_timestamp()
    );
    let member_info = match ctx.guild().unwrap().member(ctx.http(), u.id).await {
        Ok(m) => {
            let roles = get_roles_from_user(&ctx, Some(&u)).await?;
            format!(
                r#"
Display name in this server: **{name}**,
Joined at: <t:{timestamp}:R> (<t:{timestamp}:F>)
Roles: {roles}
Permissions: `{permissions}`"#,
                name = m.display_name().into_owned(),
                timestamp = m.joined_at.unwrap().unix_timestamp(),
                roles = roles
                    .iter()
                    .map(|r| format!("<@&{}>", r.0))
                    .collect::<Vec<String>>()
                    .join(", "),
                permissions = m.permissions(ctx.cache()).unwrap()
            )
        }
        Err(_) => "⚠️ User is not a member of this server.".to_string(),
    };
    msg.edit(ctx, |s| {
        s.embed(|e| {
            e.author(|a| a.name(&u.name).icon_url(u.face()))
                .title("Detailed information about the user")
                .description(format!(
                    "# User Information\n{user_info}\n\n# Member Information\n{member_info}",
                    user_info = user_info,
                    member_info = member_info
                ))
        })
    })
    .await?;
    Ok(())
}
