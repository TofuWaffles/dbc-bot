use crate::database::config::get_config;
use crate::Context;
use crate::Error;
use dbc_bot::Region;
use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::User;
use strum::IntoEnumIterator;

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
            let member = ctx
                .guild_id()
                .unwrap()
                .member(ctx.http(), user.id)
                .await
                .unwrap();
            Ok(member.roles)
        }
        None => match ctx.author_member().await {
            Some(m) => Ok(m.roles.clone()),
            None => Err("Failed to get roles from user".into()),
        },
    }
}

pub async fn get_region_role_id(ctx: &Context<'_>, region: &Region) -> Option<u64> {
    let config = get_config(ctx, region).await;
    config
        .get_str("role")
        .map_or_else(|_| None, |s| Some(s.parse::<u64>().unwrap()))
}
