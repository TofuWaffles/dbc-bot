use crate::{Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;
#[allow(dead_code)]
pub async fn remove_en_mass(
    ctx: &Context<'_>,
    msg: ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    Ok(())
}
