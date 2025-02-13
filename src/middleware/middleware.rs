use actix_web::{dev::ServiceRequest, Error, error::ErrorBadRequest};
use serde_urlencoded;
use crate::routes::queries::{HistoryQueryParams, validate_count, validate_interval};

#[allow(dead_code)]
pub async fn validate_query_params(req: ServiceRequest) -> Result<ServiceRequest, Error> {
    let query_str = req.query_string();

    match serde_urlencoded::from_str::<HistoryQueryParams>(query_str) {
        Ok(query) => {
            if let Some(ref interval) = query.interval {
                if !validate_interval(interval) {
                    return Err(ErrorBadRequest("Invalid interval. Must be one of: 5min, hour, day, week, month, quarter, year"));
                }
            }

            if let Some(count) = query.count {
                if !validate_count(count) {
                    return Err(ErrorBadRequest("Count must be between 1 and 400"));
                }
            }
            Ok(req)
        },
        Err(_) => {
            Ok(req)
        }
    }
}
