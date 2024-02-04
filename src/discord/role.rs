use dbc_bot::Region;
use poise::serenity_prelude::RoleId;
use poise::serenity_prelude::User;
use strum::IntoEnumIterator;

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
    println!("Getting roles from user");
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
