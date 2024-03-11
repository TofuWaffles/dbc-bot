use crate::database::config::{make_config, set_config};
use crate::{Context, Error};
use dbc_bot::{Mode, Region};
use futures::StreamExt;
use mongodb::{bson::doc, bson::Document, Collection};
use std::sync::Arc;
use crate::discord::prompt::prompt;
  
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


#[derive(Debug, poise::Modal)]
#[name = "Role Selection"]
struct RoleSelection{
    #[name = "Role id"]
    #[placeholder = "Please write role id here"]
    role_id: String
}

#[derive(Debug, poise::Modal)]
#[name = "Channel"]
struct Channel{
    #[name = "Enter the ID of the channel"]
    channel_id: String,
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
                role_option(ctx, msg, mci.clone(), &collection).await?;
            }
            "channel" => {
                channel_option(ctx, msg, mci.clone(), &collection).await?;
            }
            "map" => {
                map_option(ctx, msg, mci.clone(), &collection).await?;
            }
            "bracket_channel" => {
                bracket_channel_option(ctx, msg, mci.clone(), &collection).await?;
            }
            _ => {}
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
    let registration_status = if config.get_bool("registration").unwrap_or(false) {
        "Open"
    } else {
        "Closed"
    };
    let tournament_status = if config.get_bool("tournament").unwrap_or(false) {
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
        Ok(role) => format!("<@&{}>", role),
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

    let description = format!(
        r#"
        **Registration status:** {}
        **Tournament status:** {}
        **Mode:** {}
        **Map:** {}
        **Role assigned to players:** {}
        **Channel to publish results of matches:** {}
        **Channel to publish the tournament bracket:** {}
        "#,
        registration_status,
        tournament_status,
        mode,
        map,
        role,
        channel,
        bracket_channel,
    );

    msg.edit(*ctx, |s| {
        s.embed(|e| {
            e.title("Current Configuration").description(description)
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_select_menu(|m| {
                    m.custom_id("config")
                        .placeholder("Select a field to configure")
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
    mci: Arc<MessageComponentInteraction>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    match poise::execute_modal_on_component_interaction::<RoleSelection>(ctx, mci, None, None)
        .await{
        Ok(Some(role))=> {
            collection
                .update_one(doc! {}, set_config("role", Some(&role.role_id)), None)
                .await?;
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("The role has been set!").description(format!(
                        "The role **<@&{}>** has been selected for this tournament!
                        Directing back to configuration menu...",
                        role.role_id
                    ))
                })
            })
            .await?;
        
        }
        Ok(None) | Err(_) => {
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("Failed to add more role!").description(
                        "No role has been selected! Please try again!" 
                )
                })
            })
            .await?;
        }
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
        .await{
        Ok(Some(map)) => {
            collection
                .update_one(doc! {}, set_config("map", Some(map.name.as_str())), None)
                .await?;
            prompt(
                ctx,
                msg,
                "Map has been set!",
                &format!(
                    "The map **{}** has been selected for this tournament!
                    Directing back to configuration menu...",
                    map.name
                ),
                None,
                None,
            ).await?;
        }
        Ok(None) | Err(_) => {
            prompt(
                ctx,
                msg,
                "Fail to set map!",
                "No map has been selected! Please try again!",
                None,
                Some(0xFF0000),
            ).await?;
            collection
                .update_one(doc! {}, set_config("map", None), None)
                .await?;
        }
    };
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}

async fn channel_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    mci: Arc<MessageComponentInteraction>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    match poise::execute_modal_on_component_interaction::<Channel>(ctx, mci, None, None)
        .await{
        Ok(Some(channel)) => {
            collection
                .update_one(doc! {}, set_config("channel", Some(channel.channel_id.as_str())), None)
                .await?;
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("The channel has been set!").description(format!(
                        "Channel **<#{}>** has been set to post announcement!
                        Directing back to configuration menu...",
                        channel.channel_id
                    ))
                })
            })
            .await?;
        }
        Ok(None) | Err(_) => {
            prompt(
                ctx,
                msg,
                "Fail to set channel has been set!",
                "No channel has been selected! Please try again!",
                None,
                Some(0xFF0000),
            ).await?;
            collection
                .update_one(doc! {}, set_config("channel", None), None)
                .await?;
        }
    };
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}

async fn bracket_channel_option(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    mci: Arc<MessageComponentInteraction>,
    collection: &Collection<Document>,
) -> Result<(), Error> {
    match poise::execute_modal_on_component_interaction::<Channel>(ctx, mci, None, None)
        .await{
        Ok(Some(channel)) => {
            collection
                .update_one(doc! {}, set_config("bracket_channel", Some(channel.channel_id.as_str())), None)
                .await?;
            msg.edit(*ctx, |s| {
                s.components(|c| c).embed(|e| {
                    e.title("The channel has been set!").description(format!(
                        "Channel **<#{}>** has been set to update the bracket!
                        Directing back to configuration menu...",
                        channel.channel_id
                    ))
                })
            })
            .await?;
        }
        Ok(None) | Err(_) => {
            prompt(
                ctx,
                msg,
                "Fail to set channel has been set!",
                "No channel has been selected! Please try again!",
                None,
                Some(0xFF0000),
            ).await?;
            collection
                .update_one(doc! {}, set_config("bracket_channel", None), None)
                .await?;
        }
    };
    std::thread::sleep(std::time::Duration::from_secs(3)); //Delay to prevent discord from rate limiting
    Ok(())
}
