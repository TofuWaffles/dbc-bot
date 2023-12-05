use crate::bracket_tournament::config::get_config;
use crate::{bracket_tournament::region::Region, Context};
use strum::IntoEnumIterator;

pub async fn registration_open(ctx: Context<'_>) -> bool {
    for region in Region::iter() {
        let database = ctx.data().database.regional_databases.get(&region).unwrap();
        if get_config(database).await.get_bool("registration").unwrap() {
            return true;
        } else {
            continue;
        }
    }
    false
}
