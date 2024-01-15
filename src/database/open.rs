use super::config::get_config;
use crate::Context;
use dbc_bot::Region;
use strum::IntoEnumIterator;

pub async fn registration(ctx: &Context<'_>) -> bool {
    for region in Region::iter() {
        if get_config(ctx, &region)
            .await
            .get_bool("registration")
            .unwrap()
        {
            return true;
        } else {
            continue;
        }
    }
    false
}

pub async fn tournament(ctx: &Context<'_>, region: &Region) -> bool {
    get_config(ctx, region)
        .await
        .get_bool("tournament")
        .unwrap()
}

pub async fn all_tournaments(ctx: &Context<'_>) -> bool {
    for region in Region::iter() {
        if tournament(ctx, &region).await {
            return true;
        } else {
            continue;
        }
    }
    false
}
