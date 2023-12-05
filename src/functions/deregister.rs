pub async fn deregister(ctx: Context<'_>) -> Result<(), Error> {
  info!("Attempted to deregister user {}", ctx.author().tag());
  ctx.defer_ephemeral().await?;
  let msg: ReplyHandle<'_> = ctx.say("Checking registration status...").await?;
  let player = match player_registered(&ctx, None).await? {
      None => {
          msg.edit(ctx,|s|{
              s.reply(true)
              .embed(|e|{
                  e.title("**You have not registered!**")
                  .description("You have not registered for the tournament! If you want to register, please use the </register:1145363516325376031> command!")
              })
          }).await?;
          return Ok(());
      }
      Some(data) => data,
  };

  let region = Region::find_key(player.get("region").unwrap().as_str().unwrap()).unwrap();
  let database = ctx.data().database.regional_databases.get(&region).unwrap();
  let config = get_config(database).await;

  if !register_opened(&ctx, &msg, &config).await? {
      return Ok(());
  }

  msg.edit(ctx,|s|{
      s.components(|c|{
        c.create_action_row(|a|{
          a.create_button(|b|{
            b.label("Confirm")
            .style(ButtonStyle::Success)
            .custom_id("Confirm")
          })
          .create_button(|b|{
            b.label("Cancel")
            .style(ButtonStyle::Danger)
            .custom_id("Cancel")
          })
        })
      })
        .reply(true)
        .embed(|e|{
          e.title("**Are you sure you want to deregister?**")
          .description(format!("You are about to deregister from the tournament. Below information are what you told us!\nYour account name: **{}**\nWith your respective tag: **{}**\nAnd you are in the following region: **{}**", 
                              player.get("name").unwrap().as_str().unwrap(), 
                              player.get("tag").unwrap().as_str().unwrap(), 
                              &Region::find_key(player.get("region").unwrap().as_str().unwrap()).unwrap()) 
                      )
      })
  }).await?;

  let resp = msg.clone().into_message().await?;
  let cib = resp
      .await_component_interactions(&ctx.serenity_context().shard)
      .timeout(std::time::Duration::from_secs(120));
  let mut cic = cib.build();

  while let Some(mci) = cic.next().await {
      match mci.data.custom_id.as_str() {
          "Confirm" => {
              msg.edit(ctx,|s| {
                  s.components(|c| {c})
                      .embed(|e| {
                          e.title("**Deregistration is successful**")
                              .description(
                              "Seriously, are you leaving us? We hope to see you in the next tournament!",
                          )
                    })
              })
              .await?;
              remove_player(database, &player).await?;
              remove_role(&ctx, &msg, &config).await?;
          }
          "Cancel" => {
              msg.edit(ctx, |s| {
                  s.components(|c| c).embed(|e| {
                      e.title("**Deregistration cancelled!**")
                          .description("Thanks for staying in the tournament with us!")
                  })
              })
              .await?;
          }
          _ => {
              unreachable!("Cannot get here..");
          }
      }
  }
  Ok(())
}

async fn remove_player(database: &Database, player: &Document) -> Result<(), Error> {
  let collection = database.collection::<Document>("Players");
  collection
      .delete_one(doc! {"_id": player.get("_id")}, None)
      .await?;
  Ok(())
}

async fn remove_role(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    config: &Document,
) -> Result<(), Error> {
    let role_id = config
        .get("role")
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<u64>()
        .unwrap();
    let mut member = match ctx.author_member().await {
        Some(m) => m.deref().to_owned(),
        None => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Removing role failed! Please contact Moderators for this issue")
            })
            .await?;
            info!("Failed to assign role for <@{}>", user);
            return Err(Box::new(CustomError(format!(
                "Failed to assign role for <@{}>",
                user
            ))));
        }
    };
    match member.remove_role((*ctx).http(), role_id).await {
        Ok(_) => Ok(()),
        Err(_) => {
            let user = *ctx.author().id.as_u64();
            msg.edit(*ctx, |s| {
                s.content("Removing role failed! Please contact Moderators for this issue")
            })
            .await?;
            info!("Failed to remove role from <@{}>", user);
            Err(Box::new(CustomError(format!(
                "Failed to remove role from <@{}>",
                user
            ))))
        }
    }
}