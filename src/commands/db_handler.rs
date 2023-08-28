use futures::TryStreamExt;
use mongodb::bson::{doc, Document};
use crate::{Context, Error};

/// A moderator-only command, using required_permissions
#[poise::command(
    prefix_command,
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn get_individual_player_data(
    ctx: Context<'_>,
    #[description = "Check a player registration status by user ID here"] id: String,
) -> Result<(), Error> {
    let player_data = ctx
        .data()
        .db_client
        .database("DBC-bot")
        .collection("PlayerDB");

    let individual_player: Document = match player_data
        .find_one(
            doc! {
                "id": &id
            },
            None,
        )
        .await {
            Ok(Some(player)) => player,
            Ok(None) => {
                ctx.say("Player data not found in the database.").await?;
                return Ok(());
            }
            Err(err) => {
                return Err(Error::from(err));
            }
        };

    let name = individual_player
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Player username not found in database.");
    let tag = individual_player
        .get("tag")
        .and_then(|t| t.as_str())
        .unwrap_or("Player tag not found in database");

    ctx.channel_id()
        .send_message(&ctx, |response| {
            response
                .allowed_mentions(|a| a.replied_user(true))
                .embed(|e| e.title(format!("**{}**", name)).description(tag))
        })
        .await?;

    Ok(())
}


#[poise::command(
    prefix_command,
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn get_all_players_data(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let mut player_data: mongodb::Cursor<Document> = match ctx
        .data()
        .db_client
        .database("DBC-bot")
        .collection("PlayerDB")
        .find(None, None)
        .await {
            Ok(result) => result,
            Err(error) => match error.kind.as_ref() {
                mongodb::error::ErrorKind::Command(code) => {
                    return Err(Error::from(code.message.to_owned()));
                }
                _ => {
                    return Err(Error::from(error));
                }
            }
        };

    let player_data_pages = dashmap::DashMap::<String, Document>::new();

    while let Some(player_data_page) = player_data.try_next().await? {
        let name = player_data_page.get("name").and_then(|n| n.as_str()).unwrap_or("Username not found.");
        player_data_pages.insert(name.to_string(), player_data_page);
    }

    let page_content = player_data_pages
    .iter()
    .map(|entry| {
        let name = entry.key().clone();
        let data = entry.value().clone();
        let tag = data.get("tag").and_then(|t| t.as_str()).unwrap_or("Tag not found.");
        let region = data.get("region").and_then(|r| r.as_str()).unwrap_or("Region not found.");
        let id = data.get("id").and_then(|i| i.as_str()).unwrap_or("ID not found.");
        format!("Name: {}\nTag: {}\nRegion: {}\nID: {}\n", name, tag, region, id)
    })
    .collect::<Vec<_>>();

    poise::builtins::paginate(ctx, page_content.iter().map(|s| s.as_str()).collect::<Vec<_>>().as_slice()).await?;

    Ok(())
}