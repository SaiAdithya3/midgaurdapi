use mongodb::{Client, options::ClientOptions};
use dotenv::dotenv;
use std::env;

pub struct Mongodb;

impl Mongodb {
    pub async fn connect_to_mongodb() -> mongodb::error::Result<Client> {
        // Load environment variables from .env file
        dotenv().ok();
        
        // Get the MongoDB connection string from environment variables
        let mongodb_uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set in .env file");

        // Parse the connection string into options
        let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
        
        // Set additional options if needed
        client_options.app_name = Some("Midgaurd".to_string());

        // Create client
        let client = Client::with_options(client_options)?;
        
        
        println!("Successfully connected to MongoDB!");
        
        Ok(client)
    }

}