use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EarningsHistory {
    pub _id : ObjectId,
    pub pool: String,
    pub start_time: i64,
    pub end_time: i64,
    pub asset_depth: f64,
    pub rune_depth: f64,
    pub asset_price: f64,
    pub asset_price_usd: f64,
    pub liquidity_units: f64,
    pub members_count: f64,
    pub synth_units: f64,
    pub synth_supply: f64,
    pub units: f64,
    pub luvi: f64,
}