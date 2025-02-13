use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use crate::utils::get_seconds_per_interval;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use log::{debug, error};
use mongodb::bson::doc;

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
    let seconds_per_interval =
        get_seconds_per_interval(query.interval.as_deref().unwrap_or("hour"));
    let collection = &db.runepool_members_history;
    let mut match_stage = doc! {};
    
   
    if let Some(from) = query.from {
        match_stage.insert("start_time", doc! { "$gte": from });
    } else {
        let current_time = Utc::now().timestamp();
        let count = query.count.unwrap_or(400) as i64;
        match_stage.insert(
            "start_time",
            doc! { "$gte": current_time - (count * seconds_per_interval) },
        );
    }
    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }

    // Get the count of grouped records
    let count_pipeline = vec![
        doc! { "$match": match_stage.clone() },
        doc! {
            "$group": {
                "_id": {
                    "interval_start": {
                        "$subtract": [
                            { "$add": ["$end_time", 1] },
                            { "$mod": [{ "$subtract": ["$end_time", 1] }, seconds_per_interval]}
                        ]
                    }
                }
            }
        },
        doc! { "$count": "total" }
    ];

    let total_count = match collection.aggregate(count_pipeline, None).await {
        Ok(mut cursor) => {
            if let Some(doc) = cursor.try_next().await.map_err(|e| {
                error!("Cursor error: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to process results")
            })? {
                doc.get_i64("total").unwrap_or(0)
            } else {
                0
            }
        },
        Err(e) => {
            error!("Count error: {}", e);
            return Err(actix_web::error::ErrorInternalServerError("Failed to get total count"));
        }
    };

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(400);
    let skip = (page - 1) * limit;

    let sort_field = query.sort_by.as_deref().unwrap_or("startTime");
    let sort_order = match query.order.as_deref().unwrap_or("asc") {
        "desc" => -1,
        _ => 1,
    };


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
        doc! { "$sort": { sort_field: sort_order } },
        doc! { "$skip": skip },
        doc! { "$limit": limit },
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


    let response = doc! {
        "intervals": &intervals,
        "meta": {
            "startTime": first.get_i64("startTime").unwrap_or_default(),
            "endTime": last.get_i64("endTime").unwrap_or_default(),
            "startCount": first.get_f64("count").unwrap_or_default().to_string(),
            "endCount": last.get_f64("count").unwrap_or_default().to_string(),
            "startUnits": first.get_f64("units").unwrap_or_default().to_string(),
            "endUnits": last.get_f64("units").unwrap_or_default().to_string(),
            "pagination": {
            "currentPage": mongodb::bson::Bson::Int64(page),
                "totalPages": mongodb::bson::Bson::Int32((total_count as f64 / limit as f64).ceil() as i32),
                "totalRecords": mongodb::bson::Bson::Int64(total_count as i64),
                "limit": mongodb::bson::Bson::Int64(limit),
                "sortBy": sort_field,
                "order": if sort_order == 1 { "asc" } else { "desc" }
        }
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
