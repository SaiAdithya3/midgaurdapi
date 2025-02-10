use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use mongodb::Client as MongoClient;
use crate::services::db::Mongodb;
use crate::models::depth_price_history::DepthPriceHistory;

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "priceShiftLoss")]
    pub price_shift_loss: String,
    #[serde(rename = "luviIncrease")]
    pub luvi_increase: String,
    #[serde(rename = "startAssetDepth")]
    pub start_asset_depth: String,
    #[serde(rename = "startRuneDepth")]
    pub start_rune_depth: String,
    #[serde(rename = "startLPUnits")]
    pub start_lp_units: String,
    #[serde(rename = "startMemberCount")]
    pub start_member_count: String,
    #[serde(rename = "startSynthUnits")]
    pub start_synth_units: String,
    #[serde(rename = "endAssetDepth")]
    pub end_asset_depth: String,
    #[serde(rename = "endRuneDepth")]
    pub end_rune_depth: String,
    #[serde(rename = "endLPUnits")]
    pub end_lp_units: String,
    #[serde(rename = "endMemberCount")]
    pub end_member_count: String,
    #[serde(rename = "endSynthUnits")]
    pub end_synth_units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interval {
    #[serde(rename = "assetDepth")]
    pub asset_depth: String,
    #[serde(rename = "assetPrice")]
    pub asset_price: String,
    #[serde(rename = "assetPriceUSD")]
    pub asset_price_usd: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    #[serde(rename = "liquidityUnits")]
    pub liquidity_units: String,
    pub luvi: String,
    #[serde(rename = "membersCount")]
    pub members_count: String,
    #[serde(rename = "runeDepth")]
    pub rune_depth: String,
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "synthSupply")]
    pub synth_supply: String,
    #[serde(rename = "synthUnits")]
    pub synth_units: String,
    pub units: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PriceHistory {
    pub meta: Meta,
    pub intervals: Vec<Interval>,
}

#[allow(unused_variables)]
pub async fn store_to_db(client: &MongoClient, intervals: Vec<Interval>, pool: &str) -> Result<(), Box<dyn std::error::Error>> {
    let db = Mongodb::new(client.clone());
    let depth_collection = &db.depth_history;
    let mut success_count = 0;
    let mut error_count = 0;

    for interval in intervals {
        match DepthPriceHistory::try_from(interval) {
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

pub async fn fetch_depth_price_history(pool: &str, interval: &str, start_time: i64, mongo_client: &MongoClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_time = start_time;
    
    loop {
        let url = format!(
            "https://midgard.ninerealms.com/v2/history/depths/{}?interval={}&from={}&count=400",
            pool,
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
        
        match serde_json::from_str::<PriceHistory>(&text) {
            Ok(price_history) => {
                if price_history.intervals.is_empty() {
                    println!("No more intervals to process");
                    break;
                }
                
                println!("Number of intervals to process: {}", price_history.intervals.len());
                
                let end_time = price_history.meta.end_time.parse::<i64>()?;
                
                // Store data in MongoDB one by one
                store_to_db(mongo_client, price_history.intervals, pool).await?;
                
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