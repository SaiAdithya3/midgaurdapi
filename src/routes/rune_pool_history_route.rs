use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use crate::utils::get_seconds_per_interval;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use log::error;
use mongodb::bson::doc;

#[utoipa::path(
    get,
    path = "/api/history/runepool",
    params(
        ("interval" = Option<String>, Query, description = "Time interval (hour, day, week, etc.)"),
        ("count" = Option<i32>, Query, description = "Number of intervals"),
        ("from" = Option<i64>, Query, description = "Start timestamp"),
        ("to" = Option<i64>, Query, description = "End timestamp"),
        ("page" = Option<i64>, Query, description = "Page number"),
        ("limit" = Option<i64>, Query, description = "Records per page"),
        ("sort_by" = Option<String>, Query, description = "Field to sort by"),
        ("order" = Option<String>, Query, description = "Sort order (asc/desc)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved runepool history", body = RunepoolMembersUnitsHistory),
        (status = 404, description = "No runepool history found"),
        (status = 400, description = "Invalid request parameters")
    ),
    tag = "Rune Pool History"
)]
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

    let interval_pipeline = vec![
        doc! { "$match": match_stage.clone() },
        doc! {
            "$group": {
                "_id": {
                    "$subtract": [
                        "$end_time",
                        { "$mod": ["$end_time", seconds_per_interval] }
                    ]
                }
            }
        },
        doc! { "$group": {
            "_id": null,
            "count": { "$sum": 1 }
        }},
    ];

    let total_intervals = match collection.aggregate(interval_pipeline, None).await {
        Ok(mut cursor) => {
            if let Ok(Some(doc)) = cursor.try_next().await.map_err(|e| {
                error!("Cursor error: {}", e);
            }) {
                doc.get_i32("count").unwrap_or(0) as i64
            } else {
                0
            }
        }
        Err(e) => {
            error!("Count error: {}", e);
            return Err(actix_web::error::ErrorInternalServerError(
                "Failed to get total count",
            ));
        }
    };

    #[allow(unused_variables)]
    let total_docs = collection
        .count_documents(match_stage.clone(), None)
        .await
        .map_err(|e| {
            error!("Count error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total count")
        })?;

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
                            "$end_time",
                            { "$mod": ["$end_time", seconds_per_interval] }
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
            "startTime": "$_id.interval_start",
            "endTime": { "$add": ["$_id.interval_start", seconds_per_interval] },
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
            "error": "No runepool history found for the specified criteria",
            "status": 404,
            "meta": {
                "totalRecords": total_intervals,
                "currentPage": page,
                "limit": limit
            }
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
            "endUnits": last.get_f64("units").unwrap_or_default().to_string()
        },
        "pagination": {
            "currentPage": mongodb::bson::Bson::Int64(page),
            "totalPages": mongodb::bson::Bson::Int64((total_intervals as f64 / limit as f64).ceil() as i64),
            "totalRecords": mongodb::bson::Bson::Int64(total_intervals as i64),
            "limit": mongodb::bson::Bson::Int64(limit),
            "sortBy": sort_field,
            "order": if sort_order == 1 { "asc" } else { "desc" }
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
