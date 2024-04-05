use crate::database::config::get_config;
use crate::Context;
use crate::Error;
use dbc_bot::Region;
use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::User;
use strum::IntoEnumIterator;
use tracing::error;
pub async fn get_region_from_role(ctx: &Context<'_>, roles: Vec<RoleId>) -> Option<Region> {
    for role in roles.iter() {
        for region in Region::iter() {
            let config = get_config(ctx, &region).await;
            let role_id_from_db = config.get_str("role").unwrap().parse::<u64>().unwrap();
            match role.to_role_cached(ctx.cache()) {
                Some(role) => {
                    if role.id == role_id_from_db {
                        return Some(region);
                    }
                }
                None => return None,
            }
        }
    }
    None
}

/// Get the roles from a user
/// user: Option<&User> - The user to get the roles from. If None, it will get the roles from the author of the message.
pub async fn get_roles_from_user(
    ctx: &Context<'_>,
    user: Option<&User>,
) -> Result<Vec<RoleId>, Error> {
    match user {
        Some(user) => {
            let member = ctx.guild_id().unwrap().member(ctx.http(), user.id).await?;
            Ok(member.roles)
        }
        None => match ctx.author_member().await {
            Some(m) => Ok(m.roles.clone()),
            None => Err("Failed to get roles from user".into()),
        },
    }
}
#[allow(dead_code)]
pub async fn get_region_role_id(ctx: &Context<'_>, region: &Region) -> Option<u64> {
    let config = get_config(ctx, region).await;
    config
        .get_str("role")
        .map_or_else(|_| None, |s| Some(s.parse::<u64>().unwrap()))
}
/// Remove a role from a user
/// `user: poise::serenity_prelude::User` - The user to remove the role from
/// `config: &Document` - The config to get the role from
pub async fn remove_role(
    ctx: &Context<'_>,
    user: &poise::serenity_prelude::User,
    region: &Region,
) -> Result<(), Error> {
    let config = get_config(ctx, region).await;
    let role_id = config.get_str("role").unwrap().parse::<u64>().unwrap();
    let mut member = match ctx.guild().unwrap().member(ctx.http(), user.id.0).await {
        Ok(m) => m,
        Err(e) => {
            error!("{e}");
            return Err(format!(
                "Failed to find the user with id {}! User is not found in the server!",
                user.id.0
            )
            .into());
        }
    };
    match member.remove_role((*ctx).http(), role_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{e}");
            Err(format!(
                "Failed to remove the role from {}({})!",
                user.name, user.id.0
            )
            .into())
        }
    }
}

/// Assign a role to a user
/// `user: poise::serenity_prelude::User` - The user to assign the role to
/// `region: &Option<Region>` - The region to get the role from
pub async fn assign_role(
    ctx: &Context<'_>,
    user: &poise::serenity_prelude::User,
    region: &Option<Region>,
) -> Result<(), Error> {
    let config = get_config(ctx, &region.clone().unwrap()).await;
    let role = match config.get_str("role") {
        Ok(role) => Some(role),
        Err(e) => {
            error!("{e}");
            return Err("Failed to get the role from the database!".into());
        }
    };
    let mut member = match ctx.guild().unwrap().member(ctx.http(), user.id.0).await {
        Ok(m) => m,
        Err(e) => {
            error!("{e}");
            return Err("Failed to find the user! User is not found in the server!".into());
        }
    };
    match member
        .add_role((*ctx).http(), role.unwrap().parse::<u64>().unwrap())
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{e}");
            Err(format!(
                "Failed to remove the role from {}({})!",
                user.name, user.id.0
            )
            .into())
        }
    }
}
