use crate::{
    discord::{
        log::{Log, LogType},
        prompt::prompt,
    },
    Context, Error,
};
use dbc_bot::Region;
use futures::StreamExt;
use poise::{serenity_prelude::Message, ReplyHandle};

pub async fn test(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Testing Menu")
                .description("Select which test do you want to perform")
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("test_log")
                        .style(poise::serenity_prelude::ButtonStyle::Primary)
                        .label("Test Log")
                })
            })
        })
    })
    .await?;
    let mut cic = msg
        .clone()
        .into_message()
        .await?
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(300))
        .build();
    let mut test_message: Option<Message> = None;
    while let Some(mci) = &cic.next().await {
        match mci.data.custom_id.as_str() {
            "test_log" => {
                mci.defer(ctx.http()).await?;
                let log = Log::new(ctx, region, LogType::Test).await?;
                match log.test_log().await {
                    Ok(message) => {
                        post_test(ctx, msg, &message).await?;
                        test_message = Some(message)
                    }
                    Err(e) => {
                        prompt(ctx, msg, "ERROR", format!("{:#?}", e), None, Some(0xFF0000))
                            .await?;
                    }
                }
            }
            "delete" => match test_message {
                Some(ref message) => match message.delete(ctx).await {
                    Ok(_) => {
                        return prompt(
                            ctx,
                            msg,
                            "Successfully deleted the test message",
                            "The test message is deleted!",
                            None,
                            Some(0xFFFF0000),
                        )
                        .await;
                    }
                    Err(e) => {
                        return prompt(
                            ctx,
                            msg,
                            "ERROR",
                            format!("{:#?}", e),
                            None,
                            Some(0xFF0000),
                        )
                        .await;
                    }
                },
                None => {
                    prompt(
                        ctx,
                        msg,
                        "ERROR",
                        "No message to delete",
                        None,
                        Some(0xFF0000),
                    )
                    .await?;
                }
            },
            _ => continue,
        }
    }

    Ok(())
}

async fn post_test(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    message: &Message,
) -> Result<(), Error> {
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Successfully sent the test log")
                .description(format!(
                    "Test log has been sent to the log channel at [here]({})",
                    message.link()
                ))
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.custom_id("delete")
                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                        .label("Delete the test")
                })
            })
        })
    })
    .await?;
    Ok(())
}
