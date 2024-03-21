use crate::{
    discord::{checks::is_host, log::Log, prompt::prompt},
    Context, Error,
};
use poise::serenity_prelude::{Attachment, ChannelId, MessageId};
use tracing::log::error;
/// Update the proof of a log. This command can be used multiple times to add more images to the log.
#[poise::command(slash_command, check = "is_host", rename = "log-update")]
pub async fn update_proof(
    ctx: Context<'_>,
    #[description = "Message link of the log"] link: String,
    #[description = "The images of the proof"] files: Vec<Attachment>,
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

    let urls = files
        .iter()
        .map(|file| file.proxy_url.clone())
        .collect::<Vec<String>>();
    let message = match Log::update_proof(&ctx, channel_id, message_id, urls).await {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error updating proof: {}", e);
            return prompt(
                &ctx,
                &msg,
                "Error updating proof",
                "An error occurred while updating the proof. Please try again later.",
                None,
                Some(0xFF0000),
            )
            .await;
        }
    };
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
    let components = link.split('/').collect::<Vec<&str>>();
    let (channel_id, message_id) = (
        components[components.len() - 2].parse::<u64>()?,
        components[components.len() - 1].parse::<u64>()?,
    );
    Ok((channel_id, message_id))
}
