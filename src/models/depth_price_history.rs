use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

use crate::services::fetch_depth_price_history::Interval;

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

impl TryFrom<Interval> for DepthPriceHistory {
    type Error = Box<dyn std::error::Error>;

    fn try_from(interval: Interval) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            pool: "BTC.BTC".to_string(),
            start_time: interval.start_time.trim().parse::<i64>()?,
            end_time: interval.end_time.trim().parse::<i64>()?,
            asset_depth: interval.asset_depth.trim().parse::<f64>()?,
            rune_depth: interval.rune_depth.trim().parse::<f64>()?,
            asset_price: interval.asset_price.trim().parse::<f64>()?,
            asset_price_usd: interval.asset_price_usd.trim().parse::<f64>()?,
            liquidity_units: interval.liquidity_units.trim().parse::<f64>()?,
            members_count: interval.members_count.trim().parse::<f64>()?,
            synth_units: interval.synth_units.trim().parse::<f64>()?,
            synth_supply: interval.synth_supply.trim().parse::<f64>()?,
            units: interval.units.trim().parse::<f64>()?,
            luvi: interval.luvi.trim().parse::<f64>()?,
        })
    }
}