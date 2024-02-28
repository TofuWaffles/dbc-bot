use crate::{
    database::config::{get_config, reset_config},
    discord::prompt::prompt,
    Context, Error,
};
use dbc_bot::Region;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use poise::ReplyHandle;
use tracing::info;
const PROMPTS: [&str; 7] = [
    "<:info:1187845402163167363> Getting ready to reset the tournament...", //0
    "<a:loading:1187839622680690689> Clearing all rounds and resetting the config to default...", //1
    "<:tick:1187839626338111600> Complete! All rounds are purged and the config is at default!", //2
    "<a:loading:1187839622680690689> Removing regional roles from players...",                   //3
    "<:tick:1187839626338111600> Complete! All regional roles are removed!",                     //4
    "<a:loading:1187839622680690689> Clearing all players registration...",                      //5
    "<:tick:1187839626338111600> Complete! Registration is empty!",                              //6
];
pub async fn reset(ctx: &Context<'_>, msg: &ReplyHandle<'_>, region: &Region) -> Result<(), Error> {
    info!(
        "{} decides to reset the tournament.",
        ctx.author_member()
            .await
            .map_or_else(|| "Someone".to_string(), |m| m.user.name.clone())
    );
    prompt(
        ctx,
        msg,
        "Tournament Reset",
        format!(
            r#"
{info}
"#,
            info = PROMPTS[0]
        ),
        None,
        Some(0xFF0000),
    )
    .await?;
    let database = ctx.data().database.regional_databases.get(region).unwrap();
    let collection: Collection<Document> = database.collection("Players");
    let config = get_config(ctx, region).await;
    let role = config.get_str("role").unwrap().parse::<u64>().unwrap_or(0);
    prompt(
        ctx,
        msg,
        "Tournament Reset",
        format!(
            r#"
{info}
{first}
"#,
            info = PROMPTS[0],
            first = PROMPTS[1]
        ),
        None,
        Some(0xFF0000),
    )
    .await?;
    clear_rounds_and_reset_config(database).await?;
    prompt(
        ctx,
        msg,
        "Tournament Reset",
        format!(
            r#"
{info}
{first}
{second}
"#,
            info = PROMPTS[0],
            first = PROMPTS[2],
            second = PROMPTS[3]
        ),
        None,
        Some(0xFF0000),
    )
    .await?;
    remove_regional_roles(ctx, &collection, role).await?;
    prompt(
        ctx,
        msg,
        "Tournament Reset",
        format!(
            r#"
{info}
{first}
{second}
{third}
"#,
            info = PROMPTS[0],
            first = PROMPTS[2],
            second = PROMPTS[4],
            third = PROMPTS[5]
        ),
        None,
        Some(0xFF0000),
    )
    .await?;
    clear_all_players(&collection).await;
    prompt(
        ctx,
        msg,
        "Tournament Reset",
        format!(
            r#"
{info}
{first}
{second}
{third}
"#,
            info = PROMPTS[0],
            first = PROMPTS[2],
            second = PROMPTS[4],
            third = PROMPTS[6]
        ),
        None,
        Some(0xFF0000),
    )
    .await?;
    Ok(())
}

async fn clear_rounds_and_reset_config(database: &Database) -> Result<(), Error> {
    let collections = database.list_collection_names(None).await?;
    for collection in collections {
        if collection.starts_with("Round") {
            database
                .collection::<Document>(&collection)
                .drop(None)
                .await?;
        }
        if collection.starts_with("Config") {
            let config = reset_config();
            database
                .collection::<Document>(&collection)
                .update_one(doc! {}, config, None)
                .await?;
        }
    }
    Ok(())
}

async fn clear_all_players(collection: &Collection<Document>) {
    collection.delete_many(doc! {}, None).await.unwrap();
}

async fn remove_regional_roles(
    ctx: &Context<'_>,
    collection: &Collection<Document>,
    role_id: u64,
) -> Result<(), Error> {
    let mut cursor = collection.find(doc! {}, None).await?;
    while let Some(player) = cursor.next().await {
        let id = player?
            .get_str("discord_id")
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let mut member = match ctx.guild().unwrap().member(ctx.http(), id).await {
            Ok(m) => m,
            Err(_) => continue,
        };
        member.remove_role(ctx.http(), role_id).await?;
    }
    Ok(())
}
