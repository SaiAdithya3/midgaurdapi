use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DepthPriceHistory {
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

impl DepthPriceHistory {
    pub fn new(
        start_time: String,
        end_time: String,
        asset_depth: f64,
        rune_depth: f64,
        asset_price: f64,
        asset_price_usd: f64,
        liquidity_units: f64,
        members_count: f64,
        synth_units: f64,
        synth_supply: f64,
        units: f64,
        luvi: f64,
    ) -> Self {
        Self {
            start_time,
            end_time,
            asset_depth,
            rune_depth,
            asset_price,
            asset_price_usd,
            liquidity_units,
            members_count,
            synth_units,
            synth_supply,
            units,
            luvi,
        }
    }
}