use crate::{Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;
#[allow(dead_code)]
pub async fn remove_en_mass(
    _ctx: &Context<'_>,
    _msg: ReplyHandle<'_>,
    _region: &Region,
) -> Result<(), Error> {
    Ok(())
}
