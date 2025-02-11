use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use mongodb::Client as MongoClient;
use crate::services::db::Mongodb;
use crate::models::swaps_history::SwapsHistory;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Meta {
    pub start_time: String,
    pub end_time: String,
    pub to_asset_count: String,
    pub to_rune_count: String,
    pub to_trade_count: String,
    pub from_trade_count: String,
    pub synth_mint_count: String,
    pub synth_redeem_count: String,
    pub total_count: String,
    pub to_asset_volume: String,
    pub to_rune_volume: String,
    pub to_trade_volume: String,
    pub from_trade_volume: String,
    pub synth_mint_volume: String,
    pub synth_redeem_volume: String,
    pub total_volume: String,
    #[serde(rename="toAssetVolumeUSD")]
    pub to_asset_volume_usd: String,
    #[serde(rename="toRuneVolumeUSD")]
    pub to_rune_volume_usd: String,
    #[serde(rename="toTradeVolumeUSD")]
    pub to_trade_volume_usd: String,
    #[serde(rename="fromTradeVolumeUSD")]
    pub from_trade_volume_usd: String,
    #[serde(rename="synthMintVolumeUSD")]
    pub synth_mint_volume_usd: String,
    #[serde(rename="synthRedeemVolumeUSD")]
    pub synth_redeem_volume_usd: String,
    #[serde(rename="totalVolumeUSD")]
    pub total_volume_usd: String,
    pub to_asset_fees: String,
    pub to_rune_fees: String,
    pub to_trade_fees: String,
    pub from_trade_fees: String,
    pub synth_mint_fees: String,
    pub synth_redeem_fees: String,
    pub total_fees: String,
    pub to_asset_average_slip: String,
    pub to_rune_average_slip: String,
    pub to_trade_average_slip: String,
    pub from_trade_average_slip: String,
    pub synth_mint_average_slip: String,
    pub synth_redeem_average_slip: String,
    pub average_slip: String,
    #[serde(rename="runePriceUSD")]
    pub rune_price_usd: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Interval {
    pub start_time: String,
    pub end_time: String,
    pub to_asset_count: String,
    pub to_rune_count: String,
    pub to_trade_count: String,
    pub from_trade_count: String,
    pub synth_mint_count: String,
    pub synth_redeem_count: String,
    pub total_count: String,
    pub to_asset_volume: String,
    pub to_rune_volume: String,
    pub to_trade_volume: String,
    pub from_trade_volume: String,
    pub synth_mint_volume: String,
    pub synth_redeem_volume: String,
    pub total_volume: String,
    #[serde(rename = "toAssetVolumeUSD")]
    pub to_asset_volume_usd: String,
    #[serde(rename = "toRuneVolumeUSD")]
    pub to_rune_volume_usd: String,
    #[serde(rename = "toTradeVolumeUSD")]
    pub to_trade_volume_usd: String,
    #[serde(rename = "fromTradeVolumeUSD")]
    pub from_trade_volume_usd: String,
    #[serde(rename = "synthMintVolumeUSD")]
    pub synth_mint_volume_usd: String,
    #[serde(rename = "synthRedeemVolumeUSD")]
    pub synth_redeem_volume_usd: String,
    #[serde(rename = "totalVolumeUSD")]
    pub total_volume_usd: String,
    pub to_asset_fees: String,
    pub to_rune_fees: String,
    pub to_trade_fees: String,
    pub from_trade_fees: String,
    pub synth_mint_fees: String,
    pub synth_redeem_fees: String,
    pub total_fees: String,
    pub to_asset_average_slip: String,
    pub to_rune_average_slip: String,
    pub to_trade_average_slip: String,
    pub from_trade_average_slip: String,
    pub synth_mint_average_slip: String,
    pub synth_redeem_average_slip: String,
    pub average_slip: String,
    #[serde(rename = "runePriceUSD")]
    pub rune_price_usd: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct SwapHistory{
    pub intervals: Vec<Interval>,
    pub meta: Meta
}

#[allow(unused_variables)]
pub async fn store_to_db(client: &MongoClient, intervals: Vec<Interval>) -> Result<(), Box<dyn std::error::Error>> {
    let db = Mongodb::new(client.clone());
    let swaps_collection = &db.swaps_history;
    let mut success_count = 0;
    let mut error_count = 0;

    for interval in intervals {
        match SwapsHistory::try_from(interval) {
            Ok(depth_history) => {
                match db.insert_document(swaps_collection, depth_history).await {
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


pub async fn fetch_swaps_history(_pool: &str, interval: &str, start_time: i64, mongo_client: &MongoClient) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_time = start_time;
    
    loop {
        let url = format!(
            "https://midgard.ninerealms.com/v2/history/swaps?interval={}&from={}&count=400",
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
        
        match serde_json::from_str::<SwapHistory>(&text) {
            Ok(price_history) => {
                if price_history.intervals.is_empty() {
                    println!("No more intervals to process");
                    break;
                }
                
                println!("Number of intervals to process: {}", price_history.intervals.len());
                
                let end_time = price_history.meta.end_time.parse::<i64>()?;
                
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