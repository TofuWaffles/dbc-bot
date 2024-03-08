use crate::discord::prompt::prompt;
use crate::{Context, Error};
use dbc_bot::CustomError;
use futures::StreamExt;
use poise::serenity_prelude::{EmojiId, ReactionType};
use poise::ReplyHandle;
use std::sync::Arc;
use tracing::{error, info};
const TIMEOUT: u64 = 150;

pub struct AnnouncementData {
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    color: Option<u32>,
    channel_id: Option<u64>,
    message_id: Option<u64>,
}
#[derive(Debug, poise::Modal)]
#[name = "Create Announcement Modal"]
struct CreateAnnouncementModal {
    #[name = "Announcement title"]
    #[placeholder= "Enter the title of the announcement"]
    title: String,
    #[name = "Announcement description"]
    #[placeholder = "Enter the description of the announcement"]
    description: String,
    #[name = "Image"]
    #[placeholder = "If you have any image or gif, please display it as an URL. Leave blank if not needed."]
    image: String,
    #[name = "Embed color"]
    #[placeholder = "If you want to set the embed color, enter the color code like #ABCDEF. Leave blank if not needed."]
    color: String,
    #[name = "Announcement channel(id)"]
    #[placeholder = "Enter the ID of the channel where the announcement will be sent."]
    channel_id: String,
}
#[derive(Debug, poise::Modal)]
#[name = "Edit Announcement Modal"]
struct EditAnnouncementModal {
    #[name = "Announcement title"]
    #[placeholder= "Enter the title of the announcement"]
    title: String,
    #[name = "Announcement description"]
    #[placeholder = "Enter the description of the announcement"]
    description: String,
    #[name = "Image"]
    #[placeholder = "If you have any image or gif, please display it as an URL. Leave blank if not needed."]
    image: String,
    #[name = "Embed color"]
    #[placeholder = "If you want to change the color of the embed, please enter the hex code of the color such as #ABCDEF. Leave blank if not needed."]
    color: String,
    #[name = "Announcement channel(id)"]
    #[placeholder = "Enter the ID of the channel where the announcement will be sent."]
    channel_id: String,
    #[name = "Message ID"]
    #[placeholder = "Enter the ID of the announcement message that you want to edit."]
    message_id: String,
}

pub async fn announcement(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    let mut announcement_data = AnnouncementData {
        title: None,
        description: None,
        image: None,
        color: None,
        channel_id: None,
        message_id: None,
    };
    announcement_options(ctx, msg).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(TIMEOUT));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "create_announcement" => {
                info!("Create announcement modal");
                let modal = create_announcement_modal(ctx, mci.clone()).await?;
                match Some(modal) {
                    Some(announcement_modal) => {
                        if let Some(channel_id) = announcement_modal.channel_id {
                            match poise::serenity_prelude::id::ChannelId(channel_id)
                                .to_channel(&ctx.http())
                                .await
                            {
                                Ok(_) => {
                                    announcement_data = announcement_modal;
                                    display_confirmation(ctx, msg, &announcement_data).await?;
                                }
                                Err(e) => {
                                    msg.edit(*ctx, |s| {
                                        s.reply(true).ephemeral(true).embed(|e| {
                                            e.description(
                                                "Invalid channel ID provided.".to_string(),
                                            )
                                        })
                                    })
                                    .await?;
                                    error!("{e}");
                                    continue;
                                }
                            }
                        }
                    }

                    None => {
                        msg.edit(*ctx, |s| {
                            s.embed(|e| {
                                e.description("Failed to create announcement modal.".to_string())
                            })
                        })
                        .await?;
                        info!("Failed to create announcement modal.");
                        return Err(Box::new(CustomError(
                            "Failed to create announcement modal.".to_string(),
                        )));
                    }
                }
            }
            "edit_announcement" => match Some(edit_announcement_modal(ctx, mci.clone()).await?) {
                Some(announcement_modal) => {
                    match (announcement_modal.channel_id, announcement_modal.message_id) {
                        (Some(channel_id), Some(message_id)) => {
                            match (
                                poise::serenity_prelude::id::ChannelId(channel_id)
                                    .to_channel(&ctx.http())
                                    .await,
                                ctx.http().get_message(channel_id, message_id).await,
                            ) {
                                (Ok(_channel), Ok(_message)) => {
                                    announcement_data = announcement_modal;
                                    display_confirmation(ctx, msg, &announcement_data).await?;
                                }
                                (Err(_), _) => {
                                    msg.edit(*ctx, |s| {
                                        s.embed(|e| {
                                            e.description(
                                                "Invalid channel ID provided.".to_string(),
                                            )
                                        })
                                    })
                                    .await?;
                                    continue;
                                }
                                (_, Err(_)) => {
                                    msg.edit(*ctx, |s| {
                                            s.embed(|e| {
                                                    e.description("Invalid message ID provided in the specified channel.".to_string()
                                                )})
                                        }).await?;
                                    continue;
                                }
                            }
                        }
                        (_, _) => {}
                    }
                }
                None => {
                    msg.edit(*ctx, |s| {
                        s.embed(|e| {
                            e.description("Failed to create announcement modal.".to_string())
                        })
                    })
                    .await?;
                    info!("Failed to create announcement modal.");
                    return Err(Box::new(CustomError(
                        "Failed to create announcement modal.".to_string(),
                    )));
                }
            },
            "cancel" => {
                mci.defer(&ctx.http()).await?;
                prompt(
                    ctx,
                    msg,
                    "Announcement operation cancelled",
                    "You can return to this menu by running </index:1181542953542488205>",
                    None,
                    None,
                )
                .await?;
            }
            "confirm" => match announcement_data.message_id {
                Some(message_id) => {
                    match poise::serenity_prelude::id::ChannelId(
                        announcement_data.channel_id.unwrap(),
                    )
                    .edit_message(&ctx.http(), message_id, |m| {
                        m.embed(|e| {
                            e.title(announcement_data.title.clone().unwrap())
                                .description(announcement_data.description.clone().unwrap())
                                .color(announcement_data.color.unwrap_or(0xFFFFFFFF))
                                .image(announcement_data.image.clone().unwrap_or("".to_string()))
                        })
                    })
                    .await
                    {
                        Ok(message) => {
                            msg.edit(*ctx, |s| {
                                s.content("")
                                    .embed(|e| {
                                        e.description(format!(
                                            "Announcement successfully edited in <#{}>",
                                            message.channel_id
                                        ))
                                    })
                                    .components(|c| c)
                            })
                            .await?;
                        }
                        Err(_) => {
                            msg.edit(*ctx, |s| {
                                s.reply(true).ephemeral(true).embed(|e| {
                                    e.description("Failed to edit announcement.".to_string())
                                })
                            })
                            .await?;
                            info!("Failed to edit announcement.");
                            return Err(Box::new(CustomError(
                                "Failed to edit announcement.".to_string(),
                            )));
                        }
                    }
                }
                None => {
                    match poise::serenity_prelude::id::ChannelId(
                        announcement_data.channel_id.unwrap(),
                    )
                    .send_message(&ctx.http(), |m| {
                        m.embed(|e| {
                            e.title(announcement_data.title.clone().unwrap())
                                .description(announcement_data.description.clone().unwrap())
                                .color(announcement_data.color.unwrap_or(0xFFFFFFFF))
                                .image(announcement_data.image.clone().unwrap_or("".to_string()))
                        })
                    })
                    .await
                    {
                        Ok(message) => {
                            msg.edit(*ctx, |s| {
                                s.components(|c| c).embed(|e| {
                                    e.description(format!(
                                        "Announcement successfully posted in <#{}>",
                                        message.channel_id
                                    ))
                                    .color(0x00FF00FF_u32)
                                })
                            })
                            .await?;
                        }
                        Err(e) => {
                            msg.edit(*ctx, |s| {
                                s.embed(|e| {
                                    e.description("Failed to post announcement.".to_string())
                                        .color(0xFF000000_u32)
                                })
                            })
                            .await?;
                            error!("{e}");
                            return Ok(());
                        }
                    }
                }
            },
            _ => {
                continue;
            }
        }
    }
    Ok(())
}

async fn announcement_options(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    info!("Start to display nnouncement options");
    msg.edit(*ctx, |b| {
        b.embed(|e| {
            e.title("Create or edit an existing announcement")
                .description(
                    r#"Please choose whether to create an announcement or edit an existing announcement!
Please prepare these requirements before you proceed to one of the options:
# Create Announcement:
- `channel id`: The ID of the channel where the announcement will be sent.
# Edit Announcement:
- `channel id`: The ID of the channel where the announcement was originally sent in.
- `message id`: The ID of the announcement message that you want to edit."#
                )
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("create_announcement")
                        .label("Create Announcement")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                })
                .create_button(|b|
                    b.custom_id("edit_announcement")
                    .label("Edit Announcement")
                    .style(poise::serenity_prelude::ButtonStyle::Primary)
                )
            })
        })
    })
    .await?;
    info!("End to display nnouncement options");
    Ok(())
}

async fn display_confirmation(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    announcement_data: &AnnouncementData,
) -> Result<(), Error> {
    msg.edit(*ctx, |b| {
        b.content(format!(
            r#"**Announcement Creation Preview**.
Press confirm to send this announcement to <#{}>
<:tick:1187839626338111600>: Confirm
❌: Cancel
"#,
            announcement_data
                .channel_id
                .map_or_else(|| "Not yet set".to_string(), |id| format!("{id}"))
        ))
        .embed(|e| {
            e.title(announcement_data.title.clone().unwrap_or("".to_string()))
                .description(
                    &announcement_data
                        .description
                        .clone()
                        .unwrap_or("No description".to_string()),
                )
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("confirm")
                        .style(poise::serenity_prelude::ButtonStyle::Success)
                        .emoji(ReactionType::Custom {
                            animated: false,
                            id: EmojiId(1187839626338111600),
                            name: None,
                        })
                })
                .create_button(|b| {
                    b.custom_id("cancel")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .emoji(ReactionType::Unicode("❌".to_string()))
                })
            })
        })
    })
    .await?;
    Ok(())
}

pub async fn create_announcement_modal(
    ctx: &Context<'_>,
    mci: Arc<poise::serenity_prelude::MessageComponentInteraction>,
) -> Result<AnnouncementData, Error> {
    loop {
        let result = poise::execute_modal_on_component_interaction::<CreateAnnouncementModal>(
            ctx,
            mci.clone(),
            None,
            None,
        )
        .await?;
        match result {
            Some(data) => {
                let announcement_data = AnnouncementData {
                    title: Some(data.title),
                    description: Some(data.description),
                    image: Some(data.image),
                    color: Some(data.color[1..].parse::<u32>().unwrap()),
                    channel_id: Some(data.channel_id.parse::<u64>().unwrap()),
                    message_id: None,
                };
                return Ok(announcement_data);
            }
            None => continue,
        }
    }
}

pub async fn edit_announcement_modal(
    ctx: &Context<'_>,
    mci: Arc<poise::serenity_prelude::MessageComponentInteraction>,
) -> Result<AnnouncementData, Error> {
    loop {
        let result = poise::execute_modal_on_component_interaction::<EditAnnouncementModal>(
            ctx,
            mci.clone(),
            None,
            None,
        )
        .await?;
        match result {
            Some(data) => {
                let announcement_data = AnnouncementData {
                    title: Some(data.title),
                    description: Some(data.description),
                    image: Some(data.image),
                    color: Some(data.color[1..].parse::<u32>().unwrap()),
                    channel_id: Some(data.channel_id.parse::<u64>().unwrap()),
                    message_id: Some(data.message_id.parse::<u64>().unwrap()),
                };
                return Ok(announcement_data);
            }
            None => continue,
        }
    }
}
