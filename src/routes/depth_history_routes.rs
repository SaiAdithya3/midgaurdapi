use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use log::{debug, error};
use mongodb::bson::doc;
// use crate::models::depth_price_history::DepthPriceHistory;
use crate::utils::get_seconds_per_interval;

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
    let mut match_stage = doc! { "pool": &pool };
    if let Some(from) = query.from {
        match_stage.insert("start_time", doc! { "$gte": from });
    } else {
        let current_time = Utc::now().timestamp();
        let count = query.count.unwrap_or(400) as i64;
        match_stage.insert(
            "start_time",
            doc! {
                "$gte": current_time - (count * seconds_per_interval)
            },
        );
    }

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(400);
    let skip = (page - 1) * limit;
    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }
    let sort_field = query.sort_by.as_deref().unwrap_or("startTime");
    let sort_order = match query.order.as_deref().unwrap_or("asc") {
        "desc" => -1,
        _ => 1,
    };
    let pipeline = vec![
        doc! { "$match": match_stage.clone() },
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
    ];

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

    if intervals.is_empty() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No depth history found",
            "status": 404
        })));
    }

    let first = intervals.first().unwrap();
    let last = intervals.last().unwrap();

    let total_count = collection
        .count_documents(match_stage, None)
        .await
        .map_err(|e| {
            error!("Count error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total count")
        })?;

    // debug!("First document: {:?}", first);
    // debug!("Last document: {:?}", last);

    let response = doc! {
        "intervals": &intervals,
        "meta": {
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
