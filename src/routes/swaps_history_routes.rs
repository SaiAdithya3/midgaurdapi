use crate::database::db::Mongodb;
use crate::routes::queries::HistoryQueryParams;
use crate::utils::get_seconds_per_interval;
use actix_web::{get, web, HttpResponse, Result};
use chrono::Utc;
use futures_util::TryStreamExt;
use log::{debug, error};
use mongodb::bson::doc;

#[utoipa::path(
    get,
    path = "/api/history/swaps",
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
        (status = 200, description = "Successfully retrieved swaps history", body = SwapsHistory),
        (status = 404, description = "No swaps history found"),
        (status = 400, description = "Invalid request parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Swaps History"
)]
#[get("/api/history/swaps")]
pub async fn get_swaps_history(
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
    let collection = &db.swaps_history;

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
    // 35
    if let Some(to) = query.to {
        match_stage.insert("end_time", doc! { "$lte": to });
    }

    let total_count = collection
        .count_documents(match_stage.clone(), None)
        .await
        .map_err(|e| {
            error!("Count error: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total count")
        })?;

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
                "toAssetCount": { "$last": "$to_asset_count" },
                "toRuneCount": { "$last": "$to_rune_count" },
                "toTradeCount": { "$last": "$to_trade_count" },
                "fromTradeCount": { "$last": "$from_trade_count" },
                "synthMintCount": { "$last": "$synth_mint_count" },
                "synthRedeemCount": { "$last": "$synth_redeem_count" },
                "totalCount": { "$last": "$total_count" },
                "toAssetVolume": { "$last": "$to_asset_volume" },
                "toRuneVolume": { "$last": "$to_rune_volume" },
                "toTradeVolume": { "$last": "$to_trade_volume" },
                "fromTradeVolume": { "$last": "$from_trade_volume" },
                "synthMintVolume": { "$last": "$synth_mint_volume" },
                "synthRedeemVolume": { "$last": "$synth_redeem_volume" },
                "totalVolume": { "$last": "$total_volume" },
                "toAssetVolumeUSD": { "$last": "$to_asset_volume_usd" },
                "toRuneVolumeUSD": { "$last": "$to_rune_volume_usd" },
                "toTradeVolumeUSD": { "$last": "$to_trade_volume_usd" },
                "fromTradeVolumeUSD": { "$last": "$from_trade_volume_usd" },
                "synthMintVolumeUSD": { "$last": "$synth_mint_volume_usd" },
                "synthRedeemVolumeUSD": { "$last": "$synth_redeem_volume_usd" },
                "totalVolumeUSD": { "$last": "$total_volume_usd" },
                "toAssetFees": { "$last": "$to_asset_fees" },
                "toRuneFees": { "$last": "$to_rune_fees" },
                "toTradeFees": { "$last": "$to_trade_fees" },
                "fromTradeFees": { "$last": "$from_trade_fees" },
                "synthMintFees": { "$last": "$synth_mint_fees" },
                "synthRedeemFees": { "$last": "$synth_redeem_fees" },
                "totalFees": { "$last": "$total_fees" },
                "toAssetAverageSlip": { "$last": "$to_asset_average_slip" },
                "toRuneAverageSlip": { "$last": "$to_rune_average_slip" },
                "toTradeAverageSlip": { "$last": "$to_trade_average_slip" },
                "fromTradeAverageSlip": { "$last": "$from_trade_average_slip" },
                "synthMintAverageSlip": { "$last": "$synth_mint_average_slip" },
                "synthRedeemAverageSlip": { "$last": "$synth_redeem_average_slip" },
                "averageSlip": { "$last": "$average_slip" },
                "runePriceUSD": { "$last": "$rune_price_usd" },
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
            "toAssetCount": 1,
            "toRuneCount": 1,
            "toTradeCount": 1,
            "fromTradeCount": 1,
            "synthMintCount": 1,
            "synthRedeemCount": 1,
            "totalCount": 1,
            "toAssetVolume": 1,
            "toRuneVolume": 1,
            "toTradeVolume": 1,
            "fromTradeVolume": 1,
            "synthMintVolume": 1,
            "synthRedeemVolume": 1,
            "totalVolume": 1,
            "toAssetVolumeUSD": 1,
            "toRuneVolumeUSD": 1,
            "toTradeVolumeUSD": 1,
            "fromTradeVolumeUSD": 1,
            "synthMintVolumeUSD": 1,
            "synthRedeemVolumeUSD": 1,
            "totalVolumeUSD": 1,
            "toAssetFees": 1,
            "toRuneFees": 1,
            "toTradeFees": 1,
            "fromTradeFees": 1,
            "synthMintFees": 1,
            "synthRedeemFees": 1,
            "totalFees": 1,
            "toAssetAverageSlip": 1,
            "toRuneAverageSlip": 1,
            "toTradeAverageSlip": 1,
            "fromTradeAverageSlip": 1,
            "synthMintAverageSlip": 1,
            "synthRedeemAverageSlip": 1,
            "averageSlip": 1,
            "runePriceUSD": 1,
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

    // debug!("First document: {:?}", first);
    debug!("Last document: {:?}", last);

    let response = doc! {
        "intervals": &intervals,
        "meta": {
            "toAssetCount": first.get_i64("toAssetCount").unwrap(),
            "toRuneCount": first.get_i64("toRuneCount").unwrap(),
            "toTradeCount": first.get_i64("toTradeCount").unwrap(),
            "fromTradeCount": first.get_i64("fromTradeCount").unwrap(),
            "synthMintCount": first.get_i64("synthMintCount").unwrap(),
            "synthRedeemCount": first.get_i64("synthRedeemCount").unwrap(),
            "totalCount": first.get_i64("totalCount").unwrap(),
            "toAssetVolume": first.get_f64("toAssetVolume").unwrap(),
            "toRuneVolume": first.get_f64("toRuneVolume").unwrap(),
            "toTradeVolume": first.get_f64("toTradeVolume").unwrap(),
            "fromTradeVolume": first.get_f64("fromTradeVolume").unwrap(),
            "synthMintVolume": first.get_f64("synthMintVolume").unwrap(),
            "synthRedeemVolume": first.get_f64("synthRedeemVolume").unwrap(),
            "totalVolume": first.get_f64("totalVolume").unwrap(),
            "toAssetVolumeUSD": first.get_f64("toAssetVolumeUSD").unwrap(),
            "toRuneVolumeUSD": first.get_f64("toRuneVolumeUSD").unwrap(),
            "toTradeVolumeUSD": first.get_f64("toTradeVolumeUSD").unwrap(),
            "fromTradeVolumeUSD": first.get_f64("fromTradeVolumeUSD").unwrap(),
            "synthMintVolumeUSD": first.get_f64("synthMintVolumeUSD").unwrap(),
            "synthRedeemVolumeUSD": first.get_f64("synthRedeemVolumeUSD").unwrap(),
            "totalVolumeUSD": first.get_f64("totalVolumeUSD").unwrap(),
            "toAssetFees": first.get_f64("toAssetFees").unwrap(),
            "toRuneFees": first.get_f64("toRuneFees").unwrap(),
            "toTradeFees": first.get_f64("toTradeFees").unwrap(),
            "fromTradeFees": first.get_f64("fromTradeFees").unwrap(),
            "synthMintFees": first.get_f64("synthMintFees").unwrap(),
            "synthRedeemFees": first.get_f64("synthRedeemFees").unwrap(),
            "totalFees": first.get_f64("totalFees").unwrap(),
            "toAssetAverageSlip": first.get_f64("toAssetAverageSlip").unwrap(),
            "toRuneAverageSlip": first.get_f64("toRuneAverageSlip").unwrap(),
            "toTradeAverageSlip": first.get_f64("toTradeAverageSlip").unwrap(),
            "fromTradeAverageSlip": first.get_f64("fromTradeAverageSlip").unwrap(),
            "synthMintAverageSlip": first.get_f64("synthMintAverageSlip").unwrap(),
            "synthRedeemAverageSlip": first.get_f64("synthRedeemAverageSlip").unwrap(),
            "averageSlip": first.get_f64("averageSlip").unwrap(),
            "runePriceUSD": first.get_f64("runePriceUSD").unwrap(),
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
