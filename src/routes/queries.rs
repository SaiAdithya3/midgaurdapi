use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HistoryQueryParams {
    pub interval: Option<String>,
    pub count: Option<i32>,
    pub pool: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[allow(unused)]
pub fn validate_interval(interval: &str) -> bool {
    matches!(interval, "5min" | "hour" | "day" | "week" | "month" | "quarter" | "year")
}

#[allow(unused)]
pub fn validate_count(count: i32) -> bool {
    count > 0 && count <= 400
}