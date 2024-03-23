use crate::{Context, Error};
use dbc_bot::Region;
use mongodb::{
    bson::{doc, Bson, Document},
    Collection, Database,
};

use super::{config::get_config, find::find_round_from_config};
pub struct Count {
    database: Database,
    round_name: String,
    round_i32: i32,
    round: Collection<Document>,
    player_counts: u64,
}

impl Count {
    pub async fn new(ctx: &Context<'_>, region: &Region) -> Result<Self, Error> {
        let round = find_round_from_config(&get_config(ctx, region).await);
        let database = ctx.data().database.regional_databases.get(region).unwrap();

        let mut count = Count {
            database: database.to_owned(),
            round_name: round.clone(),
            round_i32: 0,
            round: database.collection(&round),
            player_counts: 0,
        };
        count.player_counts = count.get_player_counts().await?;
        count.round_i32 = count
            .round_name
            .split(' ')
            .nth(1)
            .unwrap_or("0")
            .parse::<i32>()?;
        Ok(count)
    }

    async fn get_player_counts(&self) -> Result<u64, Error> {
        let collection: Collection<Document> = self.database.collection("Players");
        let filter = doc! {"discord_id": {"$ne": Bson::Null}};
        let count = collection.count_documents(filter, None).await?;
        Ok(count)
    }

    pub fn get_counts_of_all_players(&self) -> u64 {
        self.player_counts
    }

    pub fn get_counts_of_rounds(&self) -> u64 {
        (self.player_counts as f64).log2().ceil() as u64
    }

    pub fn get_current_round(&self) -> u64 {
        self.round_i32 as u64
    }

    pub fn get_counts_of_matches_in_current_round(&self) -> u64 {
        (1 << (self.player_counts.checked_ilog2().unwrap_or(0) + 1)) / (1 << self.round_i32)
    }

    pub async fn get_counts_of_players_in_current_round(&self) -> Result<u64, Error> {
        let filter = doc! {"discord_id": {"$ne": Bson::Null}};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count)
    }

    pub async fn get_counts_of_players_in_next_round(&self) -> Result<u64, Error> {
        let current = self.get_counts_of_players_in_current_round().await?;
        Ok(current / 2)
    }

    // pub async fn get_counts_of_next_round(&self) -> Result<u64, Error>{
    //   let next = format!("Round {}",self.round_name.split(" ").nth(1).unwrap_or("0").parse::<i32>()?+1);
    //   let collection: Collection<Document> = self.database.collection(&next);
    //   let count = collection.count_documents(None, None).await?;
    //   Ok(count)
    // }

    pub async fn get_counts_of_matches_happened(&self) -> Result<u64, Error> {
        let filter = doc! {"battle": true};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count / 2)
    }

    pub async fn get_counts_of_matches_unhappened(&self) -> Result<u64, Error> {
        let filter = doc! {"battle": false};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count / 2)
    }

    pub async fn get_counts_of_matches_next_round(&self) -> Result<u64, Error> {
        Ok(self.get_counts_of_matches_in_current_round() / 2)
    }

    pub async fn get_counts_of_byes_in_current_round(&self) -> Result<u64, Error> {
        let filter = doc! {"discord_id": Bson::Null};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count)
    }

    pub async fn get_counts_of_advanced(&self) -> Result<u64, Error> {
        let filter = doc! {"defeated": false};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count)
    }

    pub async fn get_counts_of_eliminated(&self) -> Result<u64, Error> {
        let filter = doc! {"defeated": true};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count)
    }

    pub async fn get_counts_of_inactive(&self) -> Result<u64, Error> {
        let filter = doc! {"ready": false, "battle": false};
        let count = self.round.count_documents(filter, None).await?;
        Ok(count)
    }
}
