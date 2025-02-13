use chrono::{TimeZone, Utc};

use crate::routes::queries::HistoryQueryParams;
use mongodb::bson::doc;


pub fn format_timestamp(timestamp: i64) -> String {
    let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

pub fn get_seconds_per_interval(interval: &str) -> i64 {
    match interval {
        "5min" => 300,
        "hour" => 3600,
        "day" => 86400,
        "week" => 604800,
        "month" => 2592000,
        "quarter" => 7776000,
        "year" => 31536000,
        _ => 3600,
    }
}

pub fn build_match_stage(
    pool: Option<&str>,
    query: &HistoryQueryParams,
    seconds_per_interval: i64,
) -> Result<mongodb::bson::Document, actix_web::Error> {
    let mut match_stage = doc! {};
    if let Some(pool_name) = pool {
        match_stage.insert("pool", pool_name);
    }
    

    if let Some(from) = query.from {
        match_stage.insert("start_time", doc! { "$gte": from });
    } else {
        let current_time = Utc::now().timestamp();
        let count = query.count.unwrap_or(400) as i64;
        match_stage.insert(
            "start_time",
            doc! {
                "$gte": current_time - (count * seconds_per_interval)
            },
        );
    }

    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }

    Ok(match_stage)
}

/// Helper function to handle pagination and sorting
pub fn handle_pagination_and_sorting(
    query: &HistoryQueryParams,
) -> (i64, i64, i64, String, i32) {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(400);
    let skip = (page - 1) * limit;
    let sort_field = query.sort_by.as_deref().unwrap_or("startTime").to_string();
    let sort_order = match query.order.as_deref().unwrap_or("asc") {
        "desc" => -1,
        _ => 1,
    };
    (page, skip, limit, sort_field, sort_order)
}

// pub const ONE_HOUR_AGO = Utc::now().timestamp() - ONE_HOUR_SECS as i64;
// pub const START_TIMER = ONE_HOUR_AGO;
pub const ONE_HOUR_SECS: u64 = 3_600;
// pub const RUNEPOOL_START_TIME : i64= 1721865600;
pub const RUNEPOOL_START_TIME: i64 = 1648771200;
