use crate::{Context, Error};
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, options::ClientOptions, options::FindOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SelfRoles {
    user: String,
    option: String,
}

struct DbPooling {
    client: Client,
    client_options: ClientOptions,
}

impl DbPooling {
    #[tokio::main]
    pub async fn new(uri: String) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
        let client_options = ClientOptions::parse(&uri).await?;
        Ok(Client::with_options(client_options)?)
    }
}

async fn retrieve_self_roles_data(
    args: &[&str],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let db_uri = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set. Set it as an environment variable.");

    let client = DbPooling::new(db_uri).unwrap();

    let db = client.database("DBC-bot");

    let collection = db.collection::<SelfRoles>("selfroles");

    let filter = doc! { "guildId": args[0] };

    let find_options = FindOptions::builder()
        .sort(doc! { "user": args[1], "option": args[2] })
        .build();

    let mut cursor = collection.find(filter, find_options).await?;

    todo!()
}
