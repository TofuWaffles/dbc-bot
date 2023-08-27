use crate::bracket_tournament::api;
use crate::utils::misc::get_difficulty;
use crate::utils::misc::QuoteStripper;
use crate::{Context, Error};
use poise::serenity_prelude;

#[derive(Debug, poise::ChoiceParameter)]
pub enum Region {
    #[name = "North America & South America"]
    NASA,
    #[name = "Europe"]
    EU,
    #[name = "Asia & Oceania"]
    APAC,
}

/// Sign up for Discord Brawl Cup Tournament!
#[poise::command(slash_command, prefix_command, track_edits)]
pub async fn register(
    ctx: Context<'_>,
    #[description = "Put your player tag here (without #)"] tag: String,
    #[description = "Put your region here"] region: Region,
) -> Result<(), Error> {
    ctx.defer().await?;
    let registry_confirm: u64 = format!("{}1", ctx.id()).parse().unwrap(); //Message ID concatenates with 1 which indicates true
    let registry_cancel: u64 = format!("{}0", ctx.id()).parse().unwrap(); //Message ID concatenates with 0 which indicates false
    let endpoint = api::api_handlers::get_api_link("player", &tag);
    match api::api_handlers::request(&endpoint).await {
        Ok(player) => {
            ctx.send(|s| {
                s.components(|c| {
                        c.create_action_row(|a| {
                            a.create_button(|b| {
                                b.label("Confirm")
                                    .style(poise::serenity_prelude::ButtonStyle::Success)
                                    .custom_id(registry_confirm)
                            })
                            .create_button(|b| {
                                b.label("Cancel")
                                    .style(poise::serenity_prelude::ButtonStyle::Danger)
                                    .custom_id(registry_cancel)
                            })
                        })
                    })
                    .reply(true)
                    .ephemeral(false)
                    .embed(|e| {
                        e.author(|a| a.name(ctx.author().name.clone()))
                            .title(
                                "**".to_string()
                                    + &player["name"].to_string().strip_quote()
                                    + "("
                                    + &player["tag"].to_string().strip_quote()
                                    + ")**",
                            )
                            .description("**Please confirm this is the correct account that you are going to use during our tournament!**")
                            .thumbnail(format!(
                                "https://cdn-old.brawlify.com/profile-low/{}.png",
                                player["icon"]["id"]
                            ))
                            .field(format!("**Region**"), format!("{:?}", region), true)
                            .fields(vec![
                                ("Trophies", player["trophies"].to_string(), true),
                                (
                                    "Highest Trophies",
                                    player["highestTrophies"].to_string(),
                                    true,
                                ),
                                ("3v3 Victories", player["3vs3Victories"].to_string(), true),
                                ("Solo Victories", player["soloVictories"].to_string(), true),
                                ("Duo Victories", player["duoVictories"].to_string(), true),
                                (
                                    "Best Robo Rumble Time",
                                    get_difficulty(&player["bestRoboRumbleTime"]),
                                    true,
                                ),
                                (
                                    "Club",
                                    player["club"]["name"].to_string().strip_quote(),
                                    true,
                                ),
                            ])
                            .timestamp(ctx.created_at())
                    })
            }
          )
            .await?;

            while let Some(mci) = serenity_prelude::CollectComponentInteraction::new(ctx)
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(std::time::Duration::from_secs(120))
                .await
            {
                if mci.data.custom_id == registry_confirm.to_string() {
                    let mut confirm_prompt = mci.message.clone();
                    confirm_prompt
                        .edit(ctx, |s| {
                            s.components(|c| {
                                c.create_action_row(|a| {
                                    a.create_button(|b| {
                                        b.label("Confirm")
                                            .style(poise::serenity_prelude::ButtonStyle::Success)
                                            .custom_id(registry_confirm)
                                            .disabled(true)
                                    })
                                    .create_button(|b| {
                                        b.label("Cancel")
                                            .style(poise::serenity_prelude::ButtonStyle::Danger)
                                            .custom_id(registry_cancel)
                                            .disabled(true)
                                    })
                                })
                            })
                            .embed(|e| {
                                e.title(format!("**You have successfully registered!**"))
                                    .description(format!(
                                    "Your player tag #{} has been registered with the region {}",
                                    tag, region
                                ))
                            })
                        })
                        .await?;
                //ADD DATABASE HERE

                // THANKS, MATT
                } else {
                    let mut cancel_prompt = mci.message.clone();
                    cancel_prompt
                    .edit(ctx, |s| {
                        s.components(|c|{
                            c.create_action_row(|a| {
                                a.create_button(|b| {
                                    b.label("Confirm")
                                        .style(poise::serenity_prelude::ButtonStyle::Success)
                                        .custom_id(registry_confirm)
                                        .disabled(true)
                                })
                                .create_button(|b| {
                                    b.label("Cancel")
                                        .style(poise::serenity_prelude::ButtonStyle::Danger)
                                        .custom_id(registry_cancel)
                                        .disabled(true)
                                })
                            })
                        })
                        .embed(|e| {
                            e.title(format!("**Please try again**"))
                                .description(format!(
                                    "You have cancelled your registration for the tournament! Please try again!"
                                ))
                        })
                    })
                    .await?;
                }
                mci.create_interaction_response(ctx, |ir| {
                    ir.kind(serenity_prelude::InteractionResponseType::DeferredUpdateMessage)
                })
                .await?;
            }
        }

        Err(_) => {
            ctx.send(|s| {
                s.content("".to_string())
                    .reply(true)
                    .ephemeral(false)
                    .embed(|e| {
                        e.title(format!("**We have tried very hard to find but...**"))
                            .description(format!("No player is associated with the tag #{}", tag))
                            .field("Please try again!".to_string(), "".to_string(), true)
                    })
            })
            .await?;
        }
    }

    Ok(())
}
