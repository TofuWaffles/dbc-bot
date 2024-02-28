use crate::database::config::{make_config, set_config};
use crate::{Context, Error};
use dbc_bot::{Mode, Region};
use futures::StreamExt;
use mongodb::{bson::doc, bson::Document, Collection};
use std::sync::Arc;

use poise::{
    serenity_prelude::{CreateSelectMenuOption, MessageComponentInteraction},
    ReplyHandle,
};
use strum::IntoEnumIterator;

#[derive(Debug, poise::Modal)]
#[name = "Map is used for the tournament"]
struct TournamentMap {
    #[name = "Map Name"]
    #[placeholder = "Write map name here, or leave blank if any map is allowed"]
    name: String,
}

pub async fn configurate(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    msg.edit(*ctx, |s| {
        s.ephemeral(true).reply(true).embed(|e| {
            e.title("Awaiting to get config")
                .description("Please wait...")
        })
    })
    .await?;
    create_select_menu(ctx, msg).await?;
    display_config(ctx, msg, region).await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    while let Some(mci) = &cic.next().await {
        match mci.data.values[0].as_str() {
            "mode" => {
                mci.defer(&ctx.http()).await?;
                mode_option(ctx, msg, &collection).await?;
            }
            "role" => {
                mci.defer(&ctx.http()).await?;
                role_option(ctx, msg, &collection).await?;
            }
            "channel" => {
                mci.defer(&ctx.http()).await?;
                channel_option(ctx, msg, &collection).await?;
            }
            "map" => {
                map_option(ctx, msg, mci.clone(), &collection).await?;
            }
            "bracket_channel" => {
                mci.defer(&ctx.http()).await?;
                bracket_channel_option(ctx, msg, &collection).await?;
            }
            _ => create_select_menu(ctx, msg).await?,
        };
        display_config(ctx, msg, region).await?;
    }
    Ok(())
}

async fn display_config(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Config");
    let config = match collection.find_one(doc! {}, None).await? {
        Some(config) => config,
        None => {
            collection.insert_one(make_config(), None).await?;
            collection.find_one(doc! {}, None).await?.unwrap()
        }
    };
    let registration_status = if config.get_bool("registration").unwrap_or_else(|_| false) {
        "Open"
    } else {
        "Closed"
    };
    let tournament_status = if config.get_bool("tournament").unwrap_or_else(|_| false) {
        "Ongoing"
    } else {
        "Not yet started"
    };
    let map = config.get_str("map").unwrap_or("Any");
    let mode = match config.get_str("mode") {
        Ok(mode) => format!("{}", Mode::find_key(mode).unwrap()),
        Err(_) => "Not yet set".to_string(),
    };
    let role = match config.get_str("role") {
        Ok(role) => {
            format!("<@&{}>", role)
        }
        Err(_) => "Not yet set".to_string(),
    };
    let channel = match config.get_str("channel") {
        Ok(channel) => format!("<#{}>", channel),
        Err(_) => "Not yet set".to_string(),
    };
    let bracket_channel = match config.get_str("bracket_channel") {
        Ok(bracket_channel) => format!("<#{}>", bracket_channel),
        Err(_) => "Not yet set".to_string(),
    };
    msg.edit(*ctx, |s| {
        s.reply(true).ephemeral(true).embed(|e| {
            e.title("Current Configuration").description(format!(
                "**Registration status:** {registration_status}
                    **Tournament status:** {tournament_status}
                    **Mode:** {mode}
                    **Map:** {map}
                    **Role assigned to players:** {role}
                    **Channel to publish results of matches:** {channel}
                    **Channel to publish the tournament bracket:** {bracket_channel}",
            ))
        })
    })
    .await?;
    Ok(())
}

async fn create_select_menu(ctx: &Context<'_>, msg: &ReplyHandle<'_>) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.ephemeral(false).reply(true).components(|c| {
            c.create_action_row(|a| {
                a.create_select_menu(|m| {
                    m.custom_id("config")
                        .placeholder("Select a field to configurate")
                        .options(|o| {
                            o.create_option(|o| {
                                o.label("Mode")
                                    .value("mode")
                                    .description("Select game mode for the tournament")
                            })
                            .create_option(|o| {
                                o.label("Map")
                                    .value("map")
                                    .description("Set the map for that game mode")
                            })
                            .create_option(|o| {
                                o.label("Role").value("role").description(
                                    "Set the role to assign the players for the tournament",
                                )
                            })
                            .create_option(|o| {
                                o.label("Channel")
                                    .value("channel")
                                    .description("Set the channel to send the tournament updates")
                            })
                            .create_option(|o| {
                                o.label("Bracket Channel")
                                    .value("bracket_channel")
                                    .description("Set the channel to send the tournament bracket")
                            })
                        })
                })
            })
        })
    })
    .await?;
    Ok(())
}

async fn mode_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    msg.edit(*ctx, |s| {
        s.content("Setting the mode for the tournament!")
            .ephemeral(true)
            .components(|c| {
                c.create_action_row(|c| {
                    c.create_select_menu(|m| {
                        m.custom_id("menu")
                            .placeholder("Select a mode")
                            .options(|o| {
                                for mode in Mode::iter() {
                                    let mut option = CreateSelectMenuOption::default();
                                    option.label(mode.to_string()).value(mode.to_string());
                                    o.add_option(option);
                                }
                                o
                            })
                    })
                })
            })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    if let Some(mci2) = &cic.next().await {
        mci2.defer(ctx.http()).await?;
        let mode = Mode::find_key(mci2.data.values[0].as_str()).unwrap();
        collection
            .update_one(
                doc! {},
                set_config("mode", Some(format!("{:?}", mode).as_str())),
                None,
            )
            .await?;
        msg.edit(*ctx, |s| {
            s.components(|c| c).embed(|e| {
                e.title("Mode has been set!").description(format!(
                    "Mode has been set to {}
                    Directing back to configuration menu...",
                    mci2.data.values[0].as_str()
                ))
            })
        })
        .await?;
    }
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}

async fn role_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    let roles = ctx.guild_id().unwrap().roles(ctx.http()).await?;
    msg.edit(*ctx, |s| {
        s.content("Setting a role to assign to players!")
            .ephemeral(false)
            .components(|c| {
                c.create_action_row(|c| {
                    c.create_select_menu(|m| {
                        m.custom_id("menu")
                            .placeholder("Select a role.")
                            .options(|o| {
                                for (role_id, role) in roles.iter() {
                                    let mut option = CreateSelectMenuOption::default();
                                    option.label(role.clone().name).value(role_id.to_string());
                                    o.add_option(option);
                                }
                                o
                            })
                    })
                })
            })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    if let Some(mci2) = &cic.next().await {
        mci2.defer(ctx.http()).await?;
        let role_id = mci2.data.values[0].as_str();
        collection
            .update_one(doc! {}, set_config("role", Some(role_id)), None)
            .await?;
        msg.edit(*ctx, |s| {
            s.components(|c| c).embed(|e| {
                e.title("Role has been set!").description(format!(
                    "<@&{}> will be assigned to players when they register.
                    Directing back to configuration menu...",
                    role_id
                ))
            })
        })
        .await?;
    }
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}

async fn channel_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    let channels = ctx.guild_id().unwrap().channels(ctx.http()).await?;
    msg.edit(*ctx, |s| {
        s.content("Setting a channel to publish matches' result!")
            .ephemeral(false)
            .components(|c| {
                c.create_action_row(|c| {
                    c.create_select_menu(|m| {
                        m.custom_id("menu")
                            .placeholder("Select a channel")
                            .options(|o| {
                                for (channel_id, channel) in channels.iter() {
                                    let mut option = CreateSelectMenuOption::default();
                                    option
                                        .label(channel.clone().name)
                                        .value(channel_id.to_string());
                                    o.add_option(option);
                                }
                                o
                            })
                    })
                })
            })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    if let Some(mci2) = &cic.next().await {
        mci2.defer(ctx.http()).await?;
        let channel_id = mci2.data.values[0].as_str();
        collection
            .update_one(doc! {}, set_config("channel", Some(channel_id)), None)
            .await?;
        msg.edit(*ctx, |s| {
            s.components(|c| c).embed(|e| {
                e.title("Channel has been set!").description(format!(
                    "All tournament updates will be posted in <#{}>.
                    Directing back to configuration menu...",
                    channel_id
                ))
            })
        })
        .await?;
    }
    Ok(())
}

async fn map_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    mci: Arc<MessageComponentInteraction>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    match poise::execute_modal_on_component_interaction::<TournamentMap>(ctx, mci, None, None)
        .await?
    {
        Some(map) => {
            collection
                .update_one(doc! {}, set_config("map", Some(map.name.as_str())), None)
                .await?;
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("Map has been set!").description(format!(
                        "The map **{}** has been selected for this tournament!
                        Directing back to configuration menu...",
                        map.name
                    ))
                })
            })
            .await?;
        }
        None => {
            collection
                .update_one(doc! {}, set_config("map", None), None)
                .await?;
        }
    };
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}

async fn bracket_channel_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    let channels = ctx.guild_id().unwrap().channels(ctx.http()).await?;
    msg.edit(*ctx, |s| {
        s.content("Setting a channel to publish the tournament bracket!")
            .ephemeral(false)
            .components(|c| {
                c.create_action_row(|c| {
                    c.create_select_menu(|m| {
                        m.custom_id("menu")
                            .placeholder("Select a channel")
                            .options(|o| {
                                for (channel_id, channel) in channels.iter() {
                                    let mut option = CreateSelectMenuOption::default();
                                    option
                                        .label(channel.clone().name)
                                        .value(channel_id.to_string());
                                    o.add_option(option);
                                }
                                o
                            })
                    })
                })
            })
    })
    .await?;
    let resp = msg.clone().into_message().await?;
    let cib = resp
        .await_component_interactions(&ctx.serenity_context().shard)
        .timeout(std::time::Duration::from_secs(120));
    let mut cic = cib.build();
    if let Some(mci2) = &cic.next().await {
        mci2.defer(ctx.http()).await?;
        let channel_id = mci2.data.values[0].as_str();
        collection
            .update_one(
                doc! {},
                set_config("bracket_channel", Some(channel_id)),
                None,
            )
            .await?;
        msg.edit(*ctx, |s| {
            s.components(|c| c).embed(|e| {
                e.title("Channel has been set!").description(format!(
                    "All tournament bracket updates will be posted in <#{}>.
                    Directing back to configuration menu...",
                    channel_id
                ))
            })
        })
        .await?;
    }
    Ok(())
}
