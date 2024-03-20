use crate::{database::config::get_config, host::tournament::disqualify::Form, Context, Error};
use dbc_bot::Region;
use poise::serenity_prelude::{ChannelId, GuildChannel, Message, User};

#[derive(Debug)]
pub enum LogType {
    Info,
    Disqualify,
    Test,
}

#[derive(Debug)]
pub struct Log<'a> {
    pub channel: GuildChannel,
    pub log_type: LogType,
    pub ctx: Context<'a>,
    pub region: Region,
    pub host: User,
}

impl<'a> Log<'a> {
    const DEFAULT_DISQUALIFY: &'static str = "Player has been disqualified from the tournament.";

    pub async fn new(ctx: &Context<'a>, region: &Region, log_type: LogType) -> Result<Self, Error> {
        let channel = Self::get_channel(ctx, region).await?;
        Ok(Self {
            channel: channel,
            log_type,
            ctx: *ctx,
            region: region.clone(),
            host: ctx.author().to_owned(),
        })
    }
    async fn get_channel(ctx: &Context<'_>, region: &Region) -> Result<GuildChannel, Error> {
        let config = get_config(ctx, region).await;
        let channel_id = config.get_str("log_channel")?.parse::<u64>()?;
        let channel = match ctx
            .guild()
            .unwrap()
            .channels(ctx.http())
            .await?
            .get(&ChannelId(channel_id))
        {
            Some(channel) => channel.to_owned(),
            None => return Err("Failed to get channel".into()),
        };
        Ok(channel)
    }

    pub async fn send_disqualify_log(
        &self,
        form: &mut Form,
        round: &str,
    ) -> Result<Message, Error> {
        if form.reason.is_empty() {
            form.reason = Self::DEFAULT_DISQUALIFY.to_string();
        }
        let msg = self.channel
            .send_message(self.ctx.http(), |s| {
                s.embed(|e| {
                    e.title("DISQUALIFY")
                        .description(format!(
                            r#"
<@{user_id}>(`{user_id}`) has been disqualified from the tournament region {region} at round {round}.
Reason: {reason}
Disqualified by: <@{host_id}>(`{host_id}`).        
          "#,
                            user_id = form.user_id,
                            region = self.region,
                            reason = form.reason,
                            host_id = self.host.id.0

                        ))
                        .color(0xFF0000)
                })
            })
            .await?;
        Ok(msg)
    }

    pub async fn test_log(&self) -> Result<Message, Error> {
        let msg = self
            .channel
            .send_message(self.ctx.http(), |s| {
                s.embed(|e| {
                    e.title("TEST")
                        .description(format!(r#"Successfully sent a log here"#))
                })
            })
            .await?;
        Ok(msg)
    }
}
