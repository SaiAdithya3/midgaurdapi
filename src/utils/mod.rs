use chrono::{TimeZone, Utc};

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



// pub const ONE_HOUR_AGO = Utc::now().timestamp() - ONE_HOUR_SECS as i64;
// pub const START_TIMER = ONE_HOUR_AGO;
pub const ONE_HOUR_SECS: u64 = 3_600;
// pub const RUNEPOOL_START_TIME : i64= 1721865600; 
pub const RUNEPOOL_START_TIME : i64= 1648771200; 