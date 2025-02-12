use actix_web::{get, web, HttpResponse, Result};
use futures_util::TryStreamExt;
use log::{error, info};
use mongodb::bson::doc;

use crate::services::db::Mongodb;
use crate::models::depth_price_history::DepthPriceHistory;
use crate::routes::queries::{HistoryQueryParams, validate_count, validate_interval};

#[get("/api/history/depth/{pool}")]
pub async fn get_depth_history(
    db: web::Data<Mongodb>,
    query: web::Query<HistoryQueryParams>,
) -> Result<HttpResponse> {
    info!("Received depth history request with query: {:?}", query);
    
    if query.pool.as_deref() == Some("BTC.BTC") {
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Pool is required"
        })));
    }

    if let Some(ref interval) = query.interval {
        if !validate_interval(interval) {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid interval. Must be one of: 5min, hour, day, week, month, quarter, year"
            })));
        }
    }
    if let Some(count) = query.count {
        if !validate_count(count) {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Count must be between 1 and 400"
            })));
        }
    }

    let collection = &db.depth_history;

    if query.interval.is_none() && query.count.is_none() && 
       query.from.is_none() && query.to.is_none() && query.pool.is_none() {
        if let Some(doc) = collection.find_one(None, None).await.map_err(|e| {
            error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Database error")
        })? {
            return Ok(HttpResponse::Ok().json(doc));
        }
    }

    // Build filter
    let mut filter = doc! {};
    if let Some(from) = query.from {
        filter.insert("start_time", doc! { "$gte": from });
    }
    if let Some(to) = query.to {
        filter.insert("end_time", doc! { "$lte": to });
    }
    if let Some(ref pool) = query.pool {
        filter.insert("pool", pool);
    }

    let mut cursor = collection.find(filter, None).await.map_err(|e| {
        error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Database error")
    })?;

    let mut results: Vec<DepthPriceHistory> = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| {
        error!("Cursor error: {}", e);
        actix_web::error::ErrorInternalServerError("Cursor error")
    })? {
        results.push(doc);
    }

    if let Some(count) = query.count {
        results.truncate(count as usize);
    }

    Ok(HttpResponse::Ok().json(results))
}