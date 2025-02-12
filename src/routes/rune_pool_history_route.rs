use actix_web::{get, web, HttpResponse, Result};
use futures_util::TryStreamExt;
use log::{error, debug};
use mongodb::bson::doc;
use chrono::Utc;
use crate::routes::queries::HistoryQueryParams;
use crate::services::db::Mongodb;

fn get_seconds_per_interval(interval: &str) -> i64 {
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

#[get("/api/history/runepool")]
pub async fn get_runepool_history(
    db: web::Data<Mongodb>,
    query: web::Query<HistoryQueryParams>,
) -> Result<HttpResponse> {
    match (&query.interval, query.count) {
        (Some(_), None) | (None, Some(_)) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Both interval and count must be provided together",
                "status": 400
            })));
        }
        _ => {}
    }

    let seconds_per_interval = get_seconds_per_interval(query.interval.as_deref().unwrap_or("hour"));
    let collection = &db.runepool_members_history;

    let mut match_stage = doc! {};
    if let Some(from) = query.from {
        match_stage.insert("start_time", doc! { "$gte": from });
    } else {
        let current_time = Utc::now().timestamp();
        let count = query.count.unwrap_or(400) as i64;
        match_stage.insert("start_time", doc! {
            "$gte": current_time - (count * seconds_per_interval)
        });
    }
    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }

    let pipeline = vec![
        doc! { "$match": match_stage },
        doc! {
            "$group": {
                "_id": {
                    "interval_start": {
                        "$subtract": [
                            { "$add": ["$end_time", 1] },
                            { "$mod": [
                                { "$subtract": ["$end_time", 1] },
                                seconds_per_interval
                            ]}
                        ]
                    }
                },
                "count": { "$last": "$count" },
                "units": { "$last": "$units" },
                "depth": { "$last": "$depth" }
            }
        },
        doc! { "$project": {
            "_id": 0,
            "startTime": {
                "$subtract": [ "$_id.interval_start", { "$mod": [ "$_id.interval_start", seconds_per_interval ] }]
            },
            "endTime": {
                "$add": [
                    { "$subtract": [ "$_id.interval_start", { "$mod": [ "$_id.interval_start", seconds_per_interval ] }] },
                    seconds_per_interval
                ]
            },
            "count": 1,
            "units": 1,
            "depth": 1
        }},
        doc! { "$sort": { "startTime": 1 } },
        doc! { "$limit": query.count.unwrap_or(400) as i64 }
    ];

    let mut cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
        error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch runepool history")
    })?;

    let mut intervals = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| {
        error!("Cursor error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to process results")
    })? {
        intervals.push(doc);
    }

    if intervals.is_empty() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No runepool history found",
            "status": 404
        })));
    }

    let first = intervals.first().unwrap();
    let last = intervals.last().unwrap();
    
    debug!("First document: {:?}", first);
    debug!("Last document: {:?}", last);

    let response = doc! {
        "intervals": &intervals,
        "meta": {
            "startTime": first.get_i64("startTime").unwrap_or_default(),
            "endTime": last.get_i64("endTime").unwrap_or_default(),
            "startCount": first.get_f64("count").unwrap_or_default().to_string(),
            "endCount": last.get_f64("count").unwrap_or_default().to_string(),
            "startUnits": first.get_f64("units").unwrap_or_default().to_string(),
            "endUnits": last.get_f64("units").unwrap_or_default().to_string()
        }
    };

    Ok(HttpResponse::Ok().json(response))
}