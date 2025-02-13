use crate::database::db::Mongodb;
use crate::models::earnings_history::{EarningsHistory, EarningsSummaryRequest};
use crate::models::earnings_history_pools::{EarningsHistoryPools, PoolEarningsRequest};
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::Client as MongoClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    pub pool: String,
    pub asset_liquidity_fees: String,
    pub rune_liquidity_fees: String,
    pub total_liquidity_fees_rune: String,
    pub saver_earning: String,
    pub rewards: String,
    pub earnings: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub liquidity_fees: String,
    pub block_rewards: String,
    #[serde(rename = "blockEarnings")]
    pub block_earnings: Option<String>,
    pub liquidity_earnings: String,
    pub avg_node_count: String,
    #[serde(rename = "runePriceUSD")]
    pub rune_price_usd: Option<String>,
    pub pools: Vec<Pool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interval {
    #[serde(rename = "startTime")]
    pub start_time: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub liquidity_fees: String,
    pub block_rewards: String,
    #[serde(rename = "blockEarnings")]
    pub block_earnings: Option<String>,
    pub liquidity_earnings: String,
    pub avg_node_count: String,
    pub rune_price_usd: Option<String>,
    pub pools: Vec<Pool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub meta: Meta,
    pub intervals: Vec<Interval>,
}

pub async fn store_to_db(
    client: &MongoClient,
    intervals: Vec<Interval>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = Mongodb::new(client.clone());
    let earnings_collection = &db.earnings_history;
    let pools_collection = &db.earnings_history_pools;
    let mut success_count = 0;
    let mut error_count = 0;
    let mut pools_success = 0;
    let mut pools_error = 0;

    for interval in intervals {
        // First, create the earnings history entry
        let earnings_summary = EarningsSummaryRequest {
            start_time: interval.start_time.clone(),
            end_time: interval.end_time.clone(),
            block_rewards: interval.block_rewards.trim().to_string(),
            avg_node_count: interval.avg_node_count.trim().to_string(),
            bonding_earnings: interval.block_earnings.unwrap_or_default(),
            liquidity_earnings: interval.liquidity_earnings.trim().to_string(),
            liquidity_fees: interval.liquidity_fees.trim().to_string(),
            rune_price_usd: interval.rune_price_usd.unwrap_or_default(),
        };

        // Helper function to parse string to f64
        let parse_float = |s: &str| -> f64 {
            if s.trim().is_empty() {
                0.0
            } else {
                s.trim().parse::<f64>().unwrap_or(0.0)
            }
        };

        // Create earnings history with safe parsing
        let earning_history = EarningsHistory {
            _id: ObjectId::new(),
            start_time: earnings_summary
                .start_time
                .trim()
                .parse::<i64>()
                .unwrap_or(0),
            end_time: earnings_summary.end_time.trim().parse::<i64>().unwrap_or(0),
            block_rewards: parse_float(&earnings_summary.block_rewards),
            avg_node_count: parse_float(&earnings_summary.avg_node_count),
            bonding_earnings: parse_float(&earnings_summary.bonding_earnings),
            liquidity_earnings: parse_float(&earnings_summary.liquidity_earnings),
            liquidity_fees: parse_float(&earnings_summary.liquidity_fees),
            rune_price_usd: parse_float(&earnings_summary.rune_price_usd),
        };

        match db
            .insert_document(earnings_collection, earning_history)
            .await
        {
            Ok(result) => {
                success_count += 1;
                for pool in interval.pools {
                    let pool_entry = PoolEarningsRequest {
                        pool: pool.pool,
                        asset_liquidity_fees: pool.asset_liquidity_fees,
                        rune_liquidity_fees: pool.rune_liquidity_fees,
                        total_liquidity_fees_rune: pool.total_liquidity_fees_rune,
                        saver_earning: pool.saver_earning,
                        rewards: pool.rewards,
                        start_time: interval.start_time.clone(),
                        end_time: interval.end_time.clone(),
                        earnings_summary_id: result.inserted_id.as_object_id().unwrap(),
                    };

                    let pool_history = EarningsHistoryPools {
                        _id: ObjectId::new(),
                        pool: pool_entry.pool,
                        asset_liquidity_fees: parse_float(&pool_entry.asset_liquidity_fees),
                        rune_liquidity_fees: parse_float(&pool_entry.rune_liquidity_fees),
                        total_liquidity_fees_rune: parse_float(
                            &pool_entry.total_liquidity_fees_rune,
                        ),
                        saver_earning: parse_float(&pool_entry.saver_earning),
                        rewards: parse_float(&pool_entry.rewards),
                        start_time: pool_entry.start_time.trim().parse::<i64>().unwrap_or(0),
                        end_time: pool_entry.end_time.trim().parse::<i64>().unwrap_or(0),
                        earnings_summary_id: pool_entry.earnings_summary_id,
                    };

                    match db.insert_document(pools_collection, pool_history).await {
                        Ok(_) => pools_success += 1,
                        Err(e) => {
                            pools_error += 1;
                            eprintln!("Error inserting pool: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("Error inserting earnings document: {}", e);
            }
        }
    }

    println!(
        "Batch complete: {} earnings documents inserted successfully, {} failed",
        success_count, error_count
    );
    println!(
        "Pool entries: {} inserted successfully, {} failed",
        pools_success, pools_error
    );
    Ok(())
}

pub async fn fetch_earnings_history(
    interval: &str,
    start_time: i64,
    mongo_client: &MongoClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut current_time = start_time;

    loop {
        let url = format!(
            "https://midgard.ninerealms.com/v2/history/earnings?interval={}&from={}&count=400",
            // pool,
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
            Ok(price_history) => {
                if price_history.intervals.is_empty() {
                    println!("No more intervals to process");
                    break;
                }

                println!(
                    "Number of intervals to process: {}",
                    price_history.intervals.len()
                );

                let end_time = price_history.meta.end_time.parse::<i64>()?;

                // Store data in MongoDB one by one
                store_to_db(mongo_client, price_history.intervals).await?;

                let current_utc: DateTime<Utc> = Utc::now();
                let current_timestamp = current_utc.timestamp();

                if end_time >= current_timestamp {
                    break;
                }

                current_time = end_time;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Err(e) => {
                println!("Failed to parse JSON: {}", e);
                println!("Response text: {}", text);
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}
