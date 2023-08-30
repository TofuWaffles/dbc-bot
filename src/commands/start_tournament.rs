use std::cell::RefCell;
use std::rc::Rc;

use crate::bracket_tournament::models::{
    BracketPair, ParentBracketPairRef, Player, Region, TournamentStatus,
};
use crate::{Context, Error};
use chrono::Utc;
use futures::TryStreamExt;
use mongodb::bson::doc;
use tracing::{error, info, instrument, trace};


/// Starts a tournament for the given region with all the currently registered players.
#[instrument]
#[poise::command(slash_command, guild_only, rename = "start-tournament")]
pub async fn start_tournament(
    ctx: Context<'_>,
    #[description = "The region of the tournament"] region: Region,
) -> Result<(), Error> {
    // This command is currently incomplete and is horribly complicated. You have been warned.

    /*
    The general approach is to create the brackets first.
    We then create the bottom brackets (the leaf nodes)
    and we work our way up towards the root, filtering empty brackets along the way
    and collecting each node in a Vector.
    After all that, we can write can use the Vector to write all the brackets to the database.
    */
    trace!("Running start-tournament");

    info!("Getting players from the database");
    let mut players = ctx
        .data()
        .database
        .collection::<Player>("Player")
        .find(doc!("region": region.get_enum_as_string()), None)
        .await?;

    let mut players_vec = Vec::<Player>::new();

    // Converting the Cursor to a player vec has a number of benefits, mainly the ease of processing and getting the player count
    while let Some(player) = players.try_next().await? {
        info!("Adding player to players_vec");
        players_vec.push(player);
    }

    // We want to set the tournament status with this once the setup is done
    let new_tournament = TournamentStatus {
        ongoing: true,
        start_time: Some(Utc::now()),
        end_time: None,
        player_count: players_vec.len() as u32,
        region,
    };

    let current_tournament_status = match ctx
        .data()
        .database
        .collection::<TournamentStatus>("TournamentStatus")
        .find_one(doc!("region": "NASA".to_string()), None)
        .await?
    {
        Some(status) => {
            if status.ongoing == true {
                ctx.say("A tournament is already running!").await?;
                return Ok(());
            } else {
                new_tournament
            }
        }
        None => new_tournament,
    };

    if players_vec.len() < 2 {
        ctx.say("Not enough players to start a tournament!").await?;
        return Ok(());
    }

    set_up_tournament(players_vec, region);

    Ok(())
}

fn set_up_tournament(players: Vec<Player>, region: Region) -> () {
    let players_count = players.len();
    let players = Rc::new(RefCell::new(players));

    info!("Generating bracket pairs...");
    // This "root" bracket pair is a dummy, in reality, the next node down the the real root
    // The real root has an ID of the first letter of the Region.
    let root_bracket_pair = BracketPair::new("".to_string(), None, None, None, region);

    // We collect the bottom brackets to filter the empty ones later
    let bottom_brackets = Rc::new(RefCell::new(Vec::<BracketPair>::new()));

    create_bracket_pairs(
        Rc::new(RefCell::new(root_bracket_pair)),
        players.clone(),
        players_count as i32,
        region.get_enum_as_string().chars().next().unwrap(),
        region,
        bottom_brackets.clone(),
    );

    // We convert this back to a regular Vec from an Rc Refcell
    let bottom_brackets_owned = bottom_brackets.take();

    for bottom_bracket in bottom_brackets_owned.iter() {
        println!("{:?}", bottom_bracket);
    }

    let valid_brackets = process_brackets(bottom_brackets_owned);

    for bracket in valid_brackets.iter() {
        println!("{:?}", bracket);
    }

    // run process_brackets here...
}


/*
This function creates the brackets using a recursive method
and populate the bottom brackets with players.

It fills empty brackets (brackets that can't be filled by players) as an empty string and will be filtered out later;
meanwhile, nodes closer to the root are filled with None for their Player ID, as they may eventually be filled.
It's like creating a regular binary tree, execpt that the nodes point from leaf to root instead of the other way around.

This formation is necessary because we want to process the leaf node to push the winner up towards the root (finals) bracket

Note: The ID of each bracket tells you the exact position in the tree, 0 representing left and 1 representing right
For example, if the ID is "N010", it means the node is left, right, left from the root, with region (N)ASA

Note 2: This function takes an Rc<RefCell<Vec<BracketPair>>> because it is easier to deal with in a recursive function than
a normal Vec
*/
#[instrument]
fn create_bracket_pairs(
    bracket_root: ParentBracketPairRef,
    players: Rc<RefCell<Vec<Player>>>,
    player_count: i32,
    left_right: char,
    region: Region,
    bottom_brackets: Rc<RefCell<Vec<BracketPair>>>,
) {
    let current_id = bracket_root.borrow().id.clone() + left_right.to_string().as_str();
    info!(
        "Creating bracket pair {} with player count of {}",
        current_id, player_count
    );

    if player_count > 2 {
        let new_bracket = Rc::new(RefCell::new(BracketPair::new(
            current_id,
            None,
            None,
            Some(bracket_root),
            region,
        )));
        create_bracket_pairs(
            new_bracket.clone(),
            players.clone(),
            ((player_count as f32) / 2.0).ceil() as i32,
            '0',
            region,
            bottom_brackets.clone(),
        );
        create_bracket_pairs(
            new_bracket.clone(),
            players.clone(),
            ((player_count as f32) / 2.0).ceil() as i32,
            '1',
            region,
            bottom_brackets.clone(),
        );
    } else {
        let player1_id = players
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| Player {
                discord_id: "".to_string(),
                tag: "".to_string(),
                name: "".to_string(),
                region,
            })
            .discord_id
            .clone();
        let player2_id = players
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| Player {
                discord_id: "".to_string(),
                tag: "".to_string(),
                name: "".to_string(),
                region,
            })
            .discord_id
            .clone();

        let new_bracket = BracketPair::new(
            current_id,
            Some(player1_id),
            Some(player2_id),
            Some(bracket_root),
            region,
        );

        bottom_brackets.borrow_mut().push(new_bracket);
    };
}

/*
This function takes the create brackets and filters out the empty brackets.
To do this, it goes layer by layer and checks if both players are empty (but not None).

Note: This function is currently broken with an unknown unwrap case shortly after going up a layer
*/
#[instrument]
fn process_brackets(mut bottom_brackets: Vec<BracketPair>) -> Vec<BracketPair> {
    // THIS FUNCTION IS CURRENTLY BROKEN
    // Will attempt to fix this later
    let mut valid_brackets = Vec::<BracketPair>::new();
    let mut next_row = Vec::<BracketPair>::new();
    loop {
        if bottom_brackets.len() > 1 {
            let current_bracket_pair = bottom_brackets.pop().unwrap();
            if current_bracket_pair
                .player1_id
                .clone()
                .unwrap_or_else(|| ".".to_string())
                .is_empty()
                && current_bracket_pair
                    .player2_id
                    .clone()
                    .unwrap_or_else(|| ".".to_string())
                    .is_empty()
            {
                info!("Both players are empty (empty bracket)");
                let final_id_num = current_bracket_pair.id.chars().last().unwrap().to_string();
                match final_id_num.as_str() {
                    "0" => {
                        current_bracket_pair
                            .upper_bracket
                            .as_deref()
                            .unwrap()
                            .borrow_mut()
                            .player1_id = Some("".to_string())
                    }
                    "1" => {
                        current_bracket_pair
                            .upper_bracket
                            .as_deref()
                            .unwrap()
                            .borrow_mut()
                            .player2_id = Some("".to_string())
                    }
                    _ => error!(
                        "This bracket ID does not end in either 0 nor 1. ID: {}",
                        current_bracket_pair.id
                    ),
                }
                next_row.push(current_bracket_pair.upper_bracket.unwrap().take());
            } else if current_bracket_pair.player1_id.clone().is_some()
                && current_bracket_pair.player2_id.clone().is_some()
                && current_bracket_pair.player2_id.clone().unwrap().is_empty()
            {
                info!("Only player 2 is empty");
                let final_id_num = current_bracket_pair.id.chars().last().unwrap().to_string();
                match final_id_num.as_str() {
                    "0" => {
                        current_bracket_pair
                            .upper_bracket
                            .as_deref()
                            .unwrap()
                            .borrow_mut()
                            .player1_id =
                            Some(current_bracket_pair.player1_id.clone().unwrap().to_string())
                    }
                    "1" => {
                        current_bracket_pair
                            .upper_bracket
                            .as_deref()
                            .unwrap()
                            .borrow_mut()
                            .player2_id =
                            Some(current_bracket_pair.player1_id.clone().unwrap().to_string())
                    }
                    _ => error!(
                        "This bracket ID does not end in either 0 nor 1. ID: {}",
                        current_bracket_pair.id
                    ),
                }
                next_row.push(current_bracket_pair.upper_bracket.unwrap().take());
            } else {
                trace!("Both players are not empty or are None (yet to be filled)");
                next_row.push(
                    current_bracket_pair
                        .upper_bracket
                        .as_deref()
                        .unwrap()
                        .take(),
                );
                valid_brackets.push(current_bracket_pair);
            }
        } else {
            info!("Going to next layer");
            bottom_brackets = next_row.clone();

            if bottom_brackets.len() < 2 && next_row.len() < 2 {
                break;
            }

            next_row.clear();
        }
    }

    valid_brackets
}
