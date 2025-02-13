use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::services::fetch_runepool_members_units_history::Interval;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RunePoolTotalMembersHistory {
    pub _id: ObjectId,
    // pub pool: String,
    pub start_time: i64,
    pub end_time: i64,
    pub depth: Option<f64>,
    pub count: f64,
    pub units: f64,
}

impl TryFrom<Interval> for RunePoolTotalMembersHistory {
    type Error = Box<dyn std::error::Error>;

    fn try_from(interval: Interval) -> Result<Self, Self::Error> {
        Ok(Self {
            _id: ObjectId::new(),
            start_time: interval.start_time.trim().parse::<i64>()?,
            end_time: interval.end_time.trim().parse::<i64>()?,
            depth: interval.depth.and_then(|d| d.trim().parse::<f64>().ok()),
            count: interval.count.trim().parse::<f64>()?,
            units: interval.units.trim().parse::<f64>()?,
        })
    }
}
