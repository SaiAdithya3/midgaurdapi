use mongodb::{
    Client, options::ClientOptions, Collection, 
    options::InsertManyOptions, error::Error as MongoError,
    results::{InsertOneResult, InsertManyResult}
};
use dotenv::dotenv;
use std::env;
use std::time::Duration;
use std::sync::Arc;
use crate::models::{
    depth_price_history::DepthPriceHistory,
    earnings_history::EarningsHistory,
    swaps_history::SwapsHistory,
    runepool_members_units_history::RunePoolTotalMembersHistory,
    earnings_history_pools::EarningsHistoryPools,
};

#[derive(Clone)]
pub struct Mongodb {
    pub depth_history: Collection<DepthPriceHistory>,
    pub earnings_history_pools: Collection<EarningsHistoryPools>,
    pub earnings_history: Collection<EarningsHistory>,
    pub swaps_history: Collection<SwapsHistory>,
    pub runepool_members_history: Collection<RunePoolTotalMembersHistory>,
    pub client: Arc<Client>,
}

impl Mongodb {
    pub fn new(client: Client) -> Self {
        let client = Arc::new(client);
        let database = client.database("thorchain");
        Self {
            depth_history: database.collection("depth_history"),
            earnings_history_pools: database.collection("earnings_history_pools"),
            earnings_history: database.collection("earnings_history"),
            swaps_history: database.collection("swaps_history"),
            runepool_members_history: database.collection("runepool_members_history"),
            client,
        }
    }


    pub async fn insert_document<T>(&self, collection: &Collection<T>, document: T) -> Result<InsertOneResult, MongoError>
    where
        T: serde::Serialize,
    {
        collection.insert_one(document, None).await
    }

    pub async fn insert_many_documents<T>(&self, collection: &Collection<T>, documents: Vec<T>) -> Result<InsertManyResult, MongoError>
    where
        T: serde::Serialize,
    {
        let options = InsertManyOptions::builder()
            .ordered(false) 
            .build();

        collection.insert_many(documents, Some(options)).await
    }


    pub async fn connect_to_mongodb() -> Result<Client, MongoError> {
        dotenv().ok();
        
        let mongodb_uri = env::var("MONGODB_URI")
            .expect("MONGODB_URI must be set in .env file");
        
        let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
        client_options.app_name = Some("Midgaurd".to_string());
        client_options.server_selection_timeout = Some(Duration::from_secs(5));
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.max_pool_size = Some(100); // Increased for better scalability
        client_options.min_pool_size = Some(10);  // Ensure minimum connections
        
        let client = Client::with_options(client_options)?;
        
        // Test connection with timeout
        match tokio::time::timeout(
            Duration::from_secs(5),
            client.list_database_names(None, None)
        ).await {
            Ok(Ok(_)) => {
                println!("Successfully connected to MongoDB!");
                Ok(client)
            },
            Ok(Err(e)) => Err(e),
            Err(_) => Err(MongoError::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection test timed out"
            ))),
        }
    }
}