use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use mongodb::{Client as MongoClient, Collection};
use crate::services::db::Mongodb;

// static POOL: &str = "BTC.BTC";
// static apiurl: &str = "https://midgard.ninerealms.com/v2/history/depths/BTC.BTC/?interval=5min&count=400&from=1606780800";

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

pub async fn store_to_db(client: &MongoClient, intervals: Vec<Interval>) -> Result<(), Box<dyn std::error::Error>> {
    let db = Mongodb::get_depth_price_db(client).await;
    let collection: Collection<Interval> = db.collection("depth_price_intervals");
    
    use mongodb::options::IndexOptions;
    use mongodb::bson::doc;
    
    let options = IndexOptions::builder().unique(true).build();
    // collection.create_index(
    //     doc! { "startTime": 1, "endTime": 1 },
    //     Some(options)
    // ).await?;
    
    Mongodb::insert_many_documents(&collection, &intervals).await?;
    println!("Successfully stored {} intervals in database", intervals.len());
    
    Ok(())
}

pub async fn fetch_depth_price_history(pool: &str, interval: &str, start_time: i64, mongo_client: &MongoClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_time = start_time;
    let client = Client::new();

    loop {
        let url = format!(
            "https://midgard.ninerealms.com/v2/history/depths/{}?interval={}&from={}&count=400",
            pool,
            interval, 
            current_time
        );
        
        println!("Fetching URL: {}", url);
        
        let response = client.get(&url).send().await?;
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
                
                println!("Number of intervals in this batch: {}", price_history.intervals.len());
                
                // Parse end_time with better error handling
                let end_time = match price_history.meta.end_time.parse::<i64>() {
                    Ok(time) => time,
                    Err(e) => {
                        println!("Failed to parse end_time: {}", e);
                        return Err(Box::new(e));
                    }
                };

                // Store data before checking time to ensure we don't lose the last batch
                store_to_db(mongo_client, price_history.intervals).await?;
                
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