use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EarningsHistory {
    pub _id: ObjectId,
    pub start_time: i64,
    pub end_time: i64,
    pub block_rewards: f64,
    pub avg_node_count: f64,
    pub bonding_earnings: f64,
    pub liquidity_earnings: f64,
    pub liquidity_fees: f64,
    pub rune_price_usd: f64,
}

#[derive(Debug, Deserialize)]
pub struct EarningsSummaryRequest {
    pub start_time: String,
    pub end_time: String,
    pub block_rewards: String,
    pub avg_node_count: String,
    pub bonding_earnings: String,
    pub liquidity_earnings: String,
    pub liquidity_fees: String,
    pub rune_price_usd: String,
}

impl TryFrom<EarningsSummaryRequest> for EarningsHistory {
    type Error = Box<dyn std::error::Error>;

    fn try_from(intervals: EarningsSummaryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            start_time: intervals.start_time.trim().parse::<i64>()?,
            end_time: intervals.end_time.trim().parse::<i64>()?,
            block_rewards: intervals.block_rewards.trim().parse::<f64>()?,
            avg_node_count: intervals.avg_node_count.trim().parse::<f64>()?,
            bonding_earnings: intervals.bonding_earnings.trim().parse::<f64>()?,
            liquidity_earnings: intervals.liquidity_earnings.trim().parse::<f64>()?,
            liquidity_fees: intervals.liquidity_fees.trim().parse::<f64>()?,
            rune_price_usd: intervals.rune_price_usd.trim().parse::<f64>()?,
        })
    }
}
