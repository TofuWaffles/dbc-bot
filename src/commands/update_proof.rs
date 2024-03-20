use crate::{
    discord::{checks::is_host, log::Log, prompt::prompt},
    Context, Error,
};
use poise::serenity_prelude::{Attachment, ChannelId, MessageId};
use reqwest::Url;
#[poise::command(slash_command, check = "is_host")]
pub async fn update_proof(
    ctx: Context<'_>,
    #[description = "Message link of the log"] link: String,
    #[description = "The image of the"] file: Attachment,
) -> Result<(), Error> {
    let msg = ctx
        .send(|s| {
            s.reply(true).ephemeral(true).embed(|e| {
                e.title("Uploading evidence")
                    .description("Hold a second...")
            })
        })
        .await?;
    let (channel_id, message_id) = match analyse_link(link.as_str()).await {
        Ok((channel_id, message_id)) => (ChannelId(channel_id), MessageId(message_id)),
        Err(_) => {
            return prompt(
                &ctx,
                &msg,
                "Invalid link",
                "The link you provided is invalid. Please provide a valid link.",
                None,
                Some(0xFFFF00),
            )
            .await
        }
    };
    let url = Url::parse(&file.proxy_url)?;
    let message = Log::update_proof(&ctx, channel_id, message_id, url).await?;
    msg.edit(ctx, |m| {
        m.embed(|e| {
            e.title("Evidence uploaded").description(format!(
                "The evidence has been uploaded to the log. [Here]({})",
                message.link()
            ))
        })
    })
    .await?;
    Ok(())
}

async fn analyse_link(link: &str) -> Result<(u64, u64), Error> {
    let components = link.split("/").collect::<Vec<&str>>();
    let (channel_id, message_id) = (
        components[components.len() - 2].parse::<u64>()?,
        components[components.len() - 1].parse::<u64>()?,
    );
    Ok((channel_id, message_id))
}
