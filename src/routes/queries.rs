use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct HistoryQueryParams {
    /// Time interval for data grouping (5min, hour, day, week, month, quarter, year)
    pub interval: Option<String>,
    
    /// Number of records to return (1-400)
    #[param(minimum = 1, maximum = 400)]
    pub count: Option<i32>,
    
    /// Start timestamp
    pub from: Option<i64>,
    
    /// End timestamp
    pub to: Option<i64>,
    
    /// Page number for pagination
    #[param(minimum = 1)]
    pub page: Option<i64>,
    
    /// Records per page
    #[param(minimum = 1, maximum = 400)]
    pub limit: Option<i64>,
    
    /// Field to sort by
    pub sort_by: Option<String>,
    
    /// Sort order (asc or desc)
    #[param(value_type = String, example = "asc")]
    pub order: Option<String>,
}

#[allow(unused)]
pub fn validate_interval(interval: &str) -> bool {
    matches!(
        interval,
        "5min" | "hour" | "day" | "week" | "month" | "quarter" | "year"
    )
}

#[allow(unused)]
pub fn validate_count(count: i32) -> bool {
    count > 0 && count <= 400
}
