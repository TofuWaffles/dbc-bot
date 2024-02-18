use dbc_bot::Region;
use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::User;
use strum::IntoEnumIterator;
use crate::database::config::get_config;
use crate::Context;

pub fn get_region_from_role(ctx: &Context<'_>, roles: Vec<RoleId>) -> Option<Region> {
    for role in roles.iter() {
        for region in Region::iter() {
            match role.to_role_cached(ctx.cache()) {
                Some(role) => {
                    if role.name.to_lowercase() == region.short().to_lowercase() {
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
pub async fn get_roles_from_user(ctx: &Context<'_>, user: Option<&User>) -> Option<Vec<RoleId>> {
    match user {
        Some(user) => {
            let member = ctx
                .guild_id()
                .unwrap()
                .member(ctx.http(), user.id)
                .await
                .unwrap();
            Some(member.roles)
        }
        None => Some((ctx.author_member().await?.roles).clone()),
    }
}

pub async fn get_region_role_id(ctx: &Context<'_>, region: &Region) -> Option<u64> {
    let config = get_config(ctx, region).await;
    config.get_str("role").map_or_else(|_|None, |s| Some(s.parse::<u64>().unwrap()))
}