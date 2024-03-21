use crate::{database::config::get_config, host::tournament::disqualify::Form, Context, Error};
use dbc_bot::{chunk, Region};
use poise::
    serenity_prelude::{
     ChannelId, CreateEmbed, Embed, GuildChannel, Message, MessageId, User
    ,
    
};

#[derive(Debug)]
pub enum LogType {
    Info,
    Disqualify,
    DisqualifyInactives,
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
            channel,
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
        let msg = self
            .channel
            .send_message(self.ctx.http(), |s| {
                s.embed(|e| {
                    e.title("DISQUALIFY")
                        .description(format!(
                            r#"
<@{user_id}>(`{user_id}`) has been disqualified from the tournament region {region} at {round}.
**Reason**: {reason}
**Disqualified by**: <@{host_id}>(`{host_id}`).        
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

    pub async fn update_proof(
        ctx: &Context<'_>,
        channel_id: ChannelId,
        message_id: MessageId,
        imgs: Vec<String>,
    ) -> Result<Message, Error> {
        let mut embeds: Vec<CreateEmbed> = vec![];
        let current_embeds = channel_id.message(ctx.http(), message_id).await?.embeds;
        embeds.push(Self::recreate_default_log_embed(
            current_embeds[0].to_owned(),
        ));
        embeds.extend(
            imgs.iter()
                .map(|img| Self::create_new_log_embed(img.to_owned()))
                .collect::<Vec<_>>(),
        );
        current_embeds[1..]
            .to_owned()
            .iter()
            .map(|embed| Self::recreate_img_log_embed(embed.to_owned()))
            .for_each(|embed| {
                if let Some(embed) = embed {
                    embeds.push(embed);
                }
            });
        let msg = channel_id
            .edit_message(ctx.http(), message_id, |m| m.add_embeds(embeds))
            .await?;
        Ok(msg)
    }

    fn create_new_log_embed(url: String) -> CreateEmbed {
        CreateEmbed::default().image(url).to_owned()
    }

    fn recreate_img_log_embed(embed: Embed) -> Option<CreateEmbed> {
        match embed.image {
            Some(img) => Some(Self::create_new_log_embed(img.url)),
            None => None,
        }
    }

    fn recreate_default_log_embed(embed: Embed) -> CreateEmbed {
        CreateEmbed::default()
            .title(embed.title.unwrap_or_default())
            .description(embed.description.unwrap_or_default())
            .color(embed.colour.unwrap_or_default())
            .to_owned()
    }

    pub async fn test_log(&self) -> Result<Message, Error> {
        let msg = self
            .channel
            .send_message(self.ctx.http(), |s| {
                s.embed(|e| {
                    e.title("TEST")
                        .description(r#"Successfully sent a log here"#.to_string())
                })
            })
            .await?;
        Ok(msg)
    }

    pub async fn disqualify_inactive_logs(&self, players: Vec<String>) -> Result<Message, Error>{
        let _embeds: Vec<CreateEmbed> = vec![]; let mut embeds: Vec<CreateEmbed> = vec![];
        let default_embed = CreateEmbed::default()
            .title("DISQUALIFY INACTIVE")
            .description(format!(r#"Due to inactivity, the following players have been disqualified from the tournament region {region}. Disqualifed by <@{host}>(`{host}`)"#, region = self.region, host = self.host.id.0))
            .color(0xFF0000)
            .timestamp(self.ctx.created_at())
            .to_owned();
        embeds.push(default_embed);
        let chunk_player = chunk::<String>(&players, 50);
        chunk_player.iter().for_each(|chunk| {
            embeds.push(CreateEmbed::default().description(chunk.join(", ")).to_owned());
        });
        let msg = self
            .channel
            .send_message(self.ctx.http(), |s| {
                s.add_embeds(embeds)
            })
            .await?;
        Ok(msg)
    }
}
