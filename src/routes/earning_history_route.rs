#![allow(unused_imports)]
use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use crate::utils::get_seconds_per_interval;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use log::{debug, error};
use mongodb::bson::doc;

#[get("/api/history/earnings")]
pub async fn get_earnings_history(
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

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(400);
    let skip = (page - 1) * limit;

    let sort_field = query.sort_by.as_deref().unwrap_or("startTime");
    let sort_order = match query.order.as_deref().unwrap_or("asc") {
        "desc" => -1,
        _ => 1,
    };

    let seconds_per_interval =
        get_seconds_per_interval(query.interval.as_deref().unwrap_or("hour"));
    let earnings_collection = &db.earnings_history;
    let pools_collection = &db.earnings_history_pools;

    let mut match_stage = doc! {};
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
    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }

    let total_count = earnings_collection
        .count_documents(match_stage.clone(), None)
        .await
        .map_err(|e| {
            error!("Count error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total count")
        })?;
    print!("Total count: {}", total_count);

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
                "blockRewards": { "$last": "$block_rewards" },
                "avgNodeCount": { "$last": "$avg_node_count" },
                "bondingEarnings": { "$last": "$bonding_earnings" },
                "liquidityEarnings": { "$last": "$liquidity_earnings" },
                "liquidityFees": { "$last": "$liquidity_fees" },
                "runePriceUSD": { "$last": "$rune_price_usd" },
                "earnings_id": { "$last": "$_id" }
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
            "blockRewards": 1,
            "avgNodeCount": 1,
            "bondingEarnings": 1,
            "liquidityEarnings": 1,
            "liquidityFees": 1,
            "runePriceUSD": 1,
            "earnings_id": 1
        }},
        doc! { "$sort": { sort_field: sort_order } },
        doc! { "$skip": skip },
        doc! { "$limit": limit },
    ];

    let mut cursor = earnings_collection
        .aggregate(pipeline, None)
        .await
        .map_err(|e| {
            error!("Database error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to fetch earnings history")
        })?;

    let mut intervals = Vec::new();
    let mut meta = doc! {};
    let mut count = 0;

    while let Some(mut doc) = cursor.try_next().await.map_err(|e| {
        error!("Cursor error: {}", e);
        actix_web::error::ErrorInternalServerError("Failed to process results")
    })? {
        for field in [
            "blockRewards",
            "avgNodeCount",
            "bondingEarnings",
            "liquidityEarnings",
            "liquidityFees",
            "runePriceUSD",
        ]
        .iter()
        {
            if let Some(value) = doc.get(field).and_then(|v| v.as_f64()) {
                let current_sum = meta.get(field).and_then(|v| v.as_f64()).unwrap_or(0.0);
                meta.insert(*field, current_sum + value);
            }
        }
        count += 1;

        if let Some(earnings_id) = doc.get("earnings_id") {
            let pools_filter = doc! {
                "earnings_summary_id": earnings_id
            };
            let mut pools = Vec::new();
            let mut pools_cursor =
                pools_collection
                    .find(pools_filter, None)
                    .await
                    .map_err(|e| {
                        error!("Database error fetching pools: {}", e);
                        actix_web::error::ErrorInternalServerError("Failed to fetch pools data")
                    })?;

            while let Some(pool_doc) = pools_cursor.try_next().await.map_err(|e| {
                error!("Cursor error for pools: {}", e);
                actix_web::error::ErrorInternalServerError("Failed to process pools results")
            })? {
                let pool_data = doc! {
                    "pool": pool_doc.pool,
                    "assetLiquidityFees": pool_doc.asset_liquidity_fees,
                    "runeLiquidityFees": pool_doc.rune_liquidity_fees,
                    "totalLiquidityFeesRune": pool_doc.total_liquidity_fees_rune,
                    "saverEarning": pool_doc.saver_earning,
                    "rewards": pool_doc.rewards
                };
                pools.push(pool_data);
            }

            doc.insert("pools", mongodb::bson::to_bson(&pools).unwrap_or_default());
        }
        doc.remove("earnings_id");
        intervals.push(doc);
    }

    if intervals.is_empty() {
        return Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No earnings history found",
            "status": 404
        })));
    }

    if count > 1 {
        for field in ["avgNodeCount", "runePriceUSD"].iter() {
            if let Some(sum) = meta.get(field).and_then(|v| v.as_f64()) {
                meta.insert(*field, sum / count as f64);
            }
        }
    }

    let first = intervals.first().unwrap();
    let last = intervals.last().unwrap();

    // debug!("First document: {:?}", first);
    // debug!("Last document: {:?}", last);

    meta.insert("startTime", first.get("startTime").unwrap());
    meta.insert("endTime", last.get("endTime").unwrap());

    let response = doc! {
        "intervals": &intervals,
        "meta": meta,
        "pagination": {
            "currentPage": mongodb::bson::Bson::Int64(page),
                "totalPages": mongodb::bson::Bson::Int32((total_count as f64 / limit as f64).ceil() as i32),
                "totalRecords": mongodb::bson::Bson::Int64(total_count as i64),
                "limit": mongodb::bson::Bson::Int64(limit),
                "sortBy": sort_field,
                "order": if sort_order == 1 { "asc" } else { "desc" }
        }
    };

    Ok(HttpResponse::Ok().json(response))
}
