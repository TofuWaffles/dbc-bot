use crate::{database::statistics::Count, discord::prompt::prompt, Context, Error};
use dbc_bot::Region;
use poise::ReplyHandle;

pub async fn statistics_information(
    ctx: &Context<'_>,
    msg: &ReplyHandle<'_>,
    region: &Region,
) -> Result<(), Error> {
    prompt(
        ctx,
        msg,
        "Statistics",
        "<a:loading:1187839622680690689> Gathering statistics...",
        None,
        None,
    )
    .await?;
    let count = Count::new(ctx, region).await?;
    let player_counts = count.get_counts_of_all_players();
    let round_counts = count.get_counts_of_rounds();
    let current = count.get_current_round();
    let matches = count.get_counts_of_matches_in_current_round();
    let player_current = count.get_counts_of_players_in_current_round().await?;
    let byes = count.get_counts_of_byes_in_current_round().await?;
    let win = count.get_counts_of_advanced().await?;
    let lose = count.get_counts_of_eliminated().await?;
    let happen = count.get_counts_of_matches_happened().await?;
    let unhappen = count.get_counts_of_matches_unhappened().await?;
    let inactive = count.get_counts_of_inactive().await?;
    let next = count.get_counts_of_players_in_next_round().await?;
    let perc_win  = 100*win.checked_div(next).unwrap_or(0);
    let perc_happen = 100*happen.checked_div(matches).unwrap_or(0);
    let perc_unhappen = 100*unhappen.checked_div(matches).unwrap_or(0);
    let perc_in = 100*inactive.checked_div(player_current).unwrap_or(0);

    prompt(
        ctx,
        msg,
        "Statistics",
        &format!(
            r#"**Region: {r} insight.**
# Overall statistics:
**👥 Players**: {player_counts}
**⚽ Rounds**: {round_counts}
# Round {current} statistics:
**⚔️ Matches**: {matches}
**👥 Players:**: {player_current}
**👋 Byes:**: {byes} (Note: disqualifed player will be replaced by bye)
**🏆 Advanced to next round**: {win} `{perc_win}`
**❌ Eliminated**: {lose} 
**🚩 Matches taken place**: {happen} `{perc_happen}`%
**🏁 Matches not yet happened**: {unhappen} `{perc_unhappen}`%
**💀 Inactive players**: {inactive} `{perc_in}`%
"#,
            r = region.full()
        ),
        None,
        0xFFFF00,
    )
    .await
}
