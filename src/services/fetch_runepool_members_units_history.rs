use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use mongodb::Client as MongoClient;
use crate::database::db::Mongodb;

use crate::models::runepool_members_units_history::RunePoolTotalMembersHistory;

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "startUnits")]
    pub start_units: String,
    #[serde(rename = "endCount")]
    pub end_count: String,
    #[serde(rename = "startCount")]
    pub start_count: String,    
    #[serde(rename = "endUnits")]
    pub end_units: String,

}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "depth")]
    pub depth: Option<String>,
    #[serde(rename = "count")]
    pub count: String,
    #[serde(rename = "units")]
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub meta: Meta,
    pub intervals: Vec<Interval>,
}

#[allow(unused_variables)]
pub async fn store_to_db(client: &MongoClient, intervals: Vec<Interval>) -> Result<(), Box<dyn std::error::Error>> {
    let db = Mongodb::new(client.clone());
    let depth_collection = &db.runepool_members_history;
    let mut success_count = 0;
    let mut error_count = 0;

    for interval in intervals {
        match RunePoolTotalMembersHistory::try_from(interval) {
            Ok(depth_history) => {
                match db.insert_document(depth_collection, depth_history).await {
                    Ok(_) => success_count += 1,
                    Err(e) => {
                        error_count += 1;
                        eprintln!("Error inserting document: {}", e);
                    }
                }
            },
            Err(e) => {
                error_count += 1;
                eprintln!("Error converting interval: {}", e);
            }
        }
    }

    println!("Batch complete: {} documents inserted successfully, {} failed", success_count, error_count);
    Ok(())
}


pub async fn fetch_runepool_members_units_history(interval: &str, start_time: i64, mongo_client: &MongoClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_time = start_time;
    
    loop {
        let url = format!(
            "https://midgard.ninerealms.com/v2/history/runepool/?interval={}&from={}&count=400",
            interval, 
            current_time
        );
        
        println!("Fetching URL: {}", url);
        
        let response = reqwest::get(&url).await?;
        println!("Response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(format!("Request failed with status: {}", response.status()).into());
        }
        
        let text = response.text().await?;
        
        match serde_json::from_str::<ApiResponse>(&text) {
            Ok(runepool_history) => {
                if runepool_history.intervals.is_empty() {
                    println!("No more intervals to process");
                    break;
                }
                
                println!("Number of intervals to process: {}", runepool_history.intervals.len());
                
                let end_time = runepool_history.meta.end_time.parse::<i64>()?;
                
                // Store data in MongoDB one by one
                store_to_db(mongo_client, runepool_history.intervals).await?;
                
                let current_utc: DateTime<Utc> = Utc::now();
                let current_timestamp = current_utc.timestamp();

                if end_time >= current_timestamp {
                    break;
                }

                current_time = end_time;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            },
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                println!("Response text: {}", text);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}