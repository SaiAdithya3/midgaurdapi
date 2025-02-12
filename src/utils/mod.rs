use chrono::{TimeZone, Utc};

pub fn format_timestamp(timestamp: i64) -> String {
    let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
