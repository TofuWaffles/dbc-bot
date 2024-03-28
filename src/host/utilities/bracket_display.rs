use crate::{
    bracket_tournament::bracket_update::update_bracket,
    database::{config::get_config},
    discord::prompt::prompt,
    Context, Error,
};
use dbc_bot::Region;
use futures::StreamExt;
use poise::ReplyHandle;

pub async fn bracket_display(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let round = get_config(ctx, region).await.get_i32("total")?;
    msg.edit(*ctx, |m| {
        m.embed(|e| {
            e.title("Bracket Display")
                .description("Which round would you like to start at?")
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_select_menu(|s| {
                    s.custom_id("round_select")
                        .placeholder("Select a round")
                        .options(|o| {
                            for i in 1..=round {
                                o.create_option(|opt| {
                                    opt.label(format!("Round {}", i)).value(i.to_string())
                                });
                            }
                            o
                        })
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
        .timeout(std::time::Duration::from_secs(120))
        .build();
    let mut start = 1;
    if let Some(mci) = cic.next().await {
        mci.defer(ctx.http()).await?;
        start = mci.data.values[0].parse::<i32>().unwrap_or(1);
    }
    prompt(
        ctx,
        msg,
        "Generating bracket image",
        "<a:loading:1187839622680690689> Please wait while the image is being generated",
        None,
        None,
    )
    .await?;
    match update_bracket(ctx, Some(region), start).await {
        Ok(_) => {
            prompt(
                ctx,
                msg,
                "Bracket",
                "Bracket has been updated",
                None,
                0xFFFF00,
            )
            .await
        }
        Err(e) => prompt(ctx, msg, "Error", &format!("Error: {}", e), None, 0xFF0000).await,
    }
}
