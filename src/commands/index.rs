use crate::database_utils::find_player::find_player;
use crate::database_utils::registration_open::registration_open;
use crate::functions::register::register_menu;
use crate::{Context, Error};
use poise::{serenity_prelude as serenity, ReplyHandle};

// Tournament all-in-one command
#[poise::command(slash_command)]
pub async fn index(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(|s| {
            s.embed(|e| {
                e.title("Menu").description(
                    "Welcome to the menu! Here you can find commands that are available for you!",
                )
            })
        })
        .await?;
    let player = find_player(ctx).await?;
    match player {
        Some(p) => {
            todo!()
        }
        None => {
            if !registration_open(ctx).await {
                msg.edit(ctx, |b| {
                    b.embed(|e| {
                        e.title("Registration")
                            .description("Registration is currently closed")
                    })
                })
                .await?;
                return Ok(());
            }
            return Ok(register_menu(&ctx, &msg).await?);
        }
    }
}

pub async fn deregister_menu(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(ctx, |b| {
        b.components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| {
                    b.label("Deregister")
                        .style(serenity::ButtonStyle::Primary)
                        .custom_id("deregister")
                })
            })
        })
    })
    .await?;
    Ok(())
}
