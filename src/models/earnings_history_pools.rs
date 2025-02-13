use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Serialize, Deserialize)]
pub struct EarningsHistoryPools {
    pub _id: ObjectId,
    pub pool: String,
    pub asset_liquidity_fees: f64,
    pub rune_liquidity_fees: f64,
    pub total_liquidity_fees_rune: f64,
    pub saver_earning: f64,
    pub rewards: f64,
    pub start_time: i64,
    pub end_time: i64,
    pub earnings_summary_id: ObjectId,
}

#[derive(Debug, Deserialize)]
pub struct PoolEarningsRequest {
    pub pool: String,
    pub asset_liquidity_fees: String,
    pub rune_liquidity_fees: String,
    pub total_liquidity_fees_rune: String,
    pub saver_earning: String,
    pub rewards: String,
    pub start_time: String,
    pub end_time: String,
    pub earnings_summary_id: ObjectId,
}

impl TryFrom<PoolEarningsRequest> for EarningsHistoryPools {
    type Error = Box<dyn std::error::Error>;

    fn try_from(pool: PoolEarningsRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            pool: pool.pool.trim().parse::<String>()?,
            asset_liquidity_fees: pool.asset_liquidity_fees.trim().parse::<f64>()?,
            rune_liquidity_fees: pool.rune_liquidity_fees.trim().parse::<f64>()?,
            total_liquidity_fees_rune: pool.total_liquidity_fees_rune.trim().parse::<f64>()?,
            saver_earning: pool.saver_earning.trim().parse::<f64>()?,
            rewards: pool.rewards.trim().parse::<f64>()?,
            start_time: pool.start_time.trim().parse::<i64>()?,
            end_time: pool.end_time.trim().parse::<i64>()?,
            earnings_summary_id: pool.earnings_summary_id,
        })
    }
}
