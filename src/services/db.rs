// use chrono::Duration;
use mongodb::{Client, options::ClientOptions, Database, Collection};
use dotenv::dotenv;
use std::env;
use std::time::Duration;  // Change this import at the top of the file

pub struct Mongodb;

impl Mongodb {
    pub async fn connect_to_mongodb() -> mongodb::error::Result<Client> {
        dotenv().ok();
        
        let mongodb_uri = env::var("MONGODB_URI")
            .expect("MONGODB_URI must be set in .env file");
        
        println!("Attempting to connect to MongoDB at: {}", mongodb_uri);
    
        let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
        client_options.app_name = Some("Midgaurd".to_string());
        // Add server selection timeout and other options
        client_options.server_selection_timeout = Some(Duration::from_secs(5));
        client_options.connect_timeout = Some(Duration::from_secs(10));
        client_options.max_pool_size = Some(10);
        
        let client = Client::with_options(client_options)?;
        
        // Test the connection with timeout
        match tokio::time::timeout(
            Duration::from_secs(5),
            client.list_database_names(None, None)
        ).await {
            Ok(Ok(_)) => println!("Successfully connected to MongoDB!"),
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(mongodb::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "Connection test timed out"
            ))),
        }
        
        Ok(client)
    }

    // Get specific databases
    pub async fn get_depth_price_db(client: &Client) -> Database {
        client.database("depth_price_history")
    }

    pub async fn get_earnings_db(client: &Client) -> Database {
        client.database("earnings_history")
    }

    pub async fn get_swaps_db(client: &Client) -> Database {
        client.database("swaps_history")
    }

    pub async fn get_rune_pool_db(client: &Client) -> Database {
        client.database("rune_pool_history")
    }

    // Generic function to insert one document
    pub async fn insert_document<T>(collection: &Collection<T>, document: T) -> mongodb::error::Result<()>
    where
        T: serde::Serialize,
    {
        collection.insert_one(document, None).await?;
        Ok(())
    }

    // Generic function to insert many documents
    pub async fn insert_many_documents<T>(collection: &Collection<T>, documents: &Vec<T>) -> mongodb::error::Result<()>
    where
        T: serde::Serialize,
    {
        collection.insert_many(documents, None).await?;
        Ok(())
    }
}