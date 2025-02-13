use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use actix_web::{get, web, HttpResponse, Result};
// use chrono::Utc;
use futures_util::TryStreamExt;
use log::error;
use mongodb::bson::{doc, Bson};
use crate::utils::{get_seconds_per_interval, build_match_stage, handle_pagination_and_sorting};

#[utoipa::path(
    get,
    path = "/api/history/depth/{pool}",
    params(
        ("pool" = String, Path, description = "Pool identifier"),
        HistoryQueryParams
    ),
    responses(
        (status = 200, description = "Successfully retrieved depth history", body = Object),
        (status = 404, description = "No depth history found"),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Depth History"
)]
#[get("/api/history/depth/{pool}")]
pub async fn get_depth_history(
    path: web::Path<String>,
    db: web::Data<Mongodb>,
    query: web::Query<HistoryQueryParams>,
) -> Result<HttpResponse> {
    let pool = path.into_inner();
    let seconds_per_interval =
        get_seconds_per_interval(query.interval.as_deref().unwrap_or("hour"));
    let collection = &db.depth_history;

    let match_stage = build_match_stage(Some(&pool), &query, seconds_per_interval)?;
    let (_page, skip, limit, sort_field, sort_order) = handle_pagination_and_sorting(&query);

    let pipeline = build_aggregation_pipeline(
        match_stage.clone(),
        seconds_per_interval,
        skip,
        limit,
        sort_field,
        sort_order,
    );

    let mut cursor = collection.aggregate(pipeline, None).await.map_err(|e| {
        error!("Database error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to fetch depth history")
    })?;

    let mut intervals = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| {
        error!("Cursor error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to process results")
    })? {
        intervals.push(doc);
    }

    // Return 404 if no data found
    if intervals.is_empty() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No depth history found",
            "status": 404
        })));
    }

    let first = intervals.first().unwrap();
    let last = intervals.last().unwrap();

    let total_count = collection
        .count_documents(match_stage.clone(), None)
        .await
        .map_err(|e| {
            error!("Count error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total count")
        })?;

    let response = doc! {
        "intervals": &intervals,
        "meta": build_meta_response(first, last, total_count as i64, query.page.unwrap_or(1), query.limit.unwrap_or(50))
    };

    Ok(HttpResponse::Ok().json(response))
}


fn build_aggregation_pipeline(
    match_stage: mongodb::bson::Document,
    seconds_per_interval: i64,
    skip: i64,
    limit: i64,
    sort_field: String,
    sort_order: i32,
) -> Vec<mongodb::bson::Document> {
    vec![
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
                "assetDepth": { "$last": "$asset_depth" },
                "assetPrice": { "$last": "$asset_price" },
                "assetPriceUSD": { "$last": "$asset_price_usd" },
                "liquidityUnits": { "$last": "$liquidity_units" },
                "luvi": { "$last": "$luvi" },
                "membersCount": { "$last": "$members_count" },
                "runeDepth": { "$last": "$rune_depth" },
                "synthSupply": { "$last": "$synth_supply" },
                "synthUnits": { "$last": "$synth_units" },
                "units": { "$last": "$units" }
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
            "assetDepth": 1,
            "assetPrice": 1,
            "assetPriceUSD": 1,
            "liquidityUnits": 1,
            "luvi": 1,
            "membersCount": 1,
            "runeDepth": 1,
            "synthSupply": 1,
            "synthUnits": 1,
            "units": 1
        }},
        doc! { "$sort": { sort_field: sort_order } },
        doc! { "$skip": skip },
        doc! { "$limit": limit },
    ]
}

/// Helper function to build the metadata response
fn build_meta_response(
    first: &mongodb::bson::Document,
    last: &mongodb::bson::Document,
    total_count: i64,
    page: i64,
    limit: i64,
) -> mongodb::bson::Document {
    doc! {
        "startTime": first.get_i64("startTime").unwrap_or_default(),
        "endTime": last.get_i64("endTime").unwrap_or_default(),
        "startAssetDepth": first.get_f64("assetDepth").unwrap_or_default().to_string(),
        "endAssetDepth": last.get_f64("assetDepth").unwrap_or_default().to_string(),
        "startRuneDepth": first.get_f64("runeDepth").unwrap_or_default().to_string(),
        "endRuneDepth": last.get_f64("runeDepth").unwrap_or_default().to_string(),
        "startLPUnits": first.get_f64("units").unwrap_or_default().to_string(),
        "endLPUnits": last.get_f64("units").unwrap_or_default().to_string(),
        "startMemberCount": first.get_i32("membersCount").unwrap_or_default(),
        "endMemberCount": last.get_i32("membersCount").unwrap_or_default(),
        "startSynthUnits": first.get_f64("synthUnits").unwrap_or_default().to_string(),
        "endSynthUnits": last.get_f64("synthUnits").unwrap_or_default().to_string(),
        "priceShiftLoss": first.get_f64("assetPrice").unwrap_or_default() - last.get_f64("assetPrice").unwrap_or_default(),
        "luviIncrease": last.get_f64("luvi").unwrap_or_default() - first.get_f64("luvi").unwrap_or_default(),
        "pagination": {
            "currentPage": Bson::Int64(page),
            "totalPages": Bson::Int32((total_count as f64 / limit as f64).ceil() as i32),
            "totalRecords": Bson::Int64(total_count),
            "limit": Bson::Int64(limit),
            "sortBy": "startTime", 
            "order": "asc"  
        }
    }
}
