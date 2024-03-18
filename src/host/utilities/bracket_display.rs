
use crate::{bracket_tournament::bracket_update::update_bracket, Context, Error};
use dbc_bot::Region;

pub async fn bracket_display(ctx: &Context<'_>, region: &Region) -> Result<(), Error> {
    return update_bracket(ctx, Some(&region)).await;
}