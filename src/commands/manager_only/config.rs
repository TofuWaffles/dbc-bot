use crate::{
  Context, 
  Error,
  bracket_tournament::config::set_config,
  bracket_tournament::region::{
    Mode,
    Region
  }
};
use mongodb::{
  bson::doc,
  Collection,
  bson::Document
};

/// Set config for the tournament
#[poise::command(slash_command, 
  guild_only,
  required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS"
)]
pub async fn config(ctx: Context<'_>, region: Region, mode: Mode, map: String) -> Result<(), Error> {
  let database = ctx.data().database.regional_databases.get(&region).unwrap();
  let collection: Collection<Document> = database.collection("Config");
  let config = set_config(&mode, &map);
  collection.update_one(doc!{}, config, None).await.unwrap();
  let post_config = collection.find_one(doc!{}, None).await.unwrap().unwrap();
  ctx.send(|s|{
    s.reply(true)
    .ephemeral(true)
    .embed(|e|{
      e.title("**Configuration has been updated!**")
      .description("The configuration for this tournament is shown below")
      .fields(vec![
        (format!("Region: {}", region),"",false),
        (format!("Mode: {}", mode),"",false),
        (format!("Map: {}", map),"",false),
        (format!("Registration: {}", post_config.get("registration").unwrap().to_string()),"",false),
      ])
    })
  }).await?;
  Ok(())
}
