use chrono::{DateTime, TimeZone, Utc};
mod services {
    pub mod db;
    pub mod fetchDepthPriceHistory;
}

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::Mutex;
use mongodb::Client;

// Wrap MongoDB client in app state
#[allow(dead_code)]
struct AppState {
    db: Mutex<Option<Client>>,
}

pub fn format_timestamp(timestamp: i64) -> String {
    let datetime = Utc.timestamp_opt(timestamp, 0).unwrap();
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}


#[actix_web::get("/")]
async fn home_route() -> impl Responder {
    HttpResponse::Ok().body("Yokoso, watashi no midgard api desu!")
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Server is running!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create MongoDB client
    let mongo_client = services::db::Mongodb::connect_to_mongodb()
        .await
        .expect("Failed to connect to MongoDB");

    // Use the client before wrapping it in AppState
    let start_time = 1704326400;
    let pool = String::from("BTC.BTC");
    let interval = String::from("hour");

    println!("Starting fetch from: {}", format_timestamp(start_time));

    if let Err(e) = services::fetchDepthPriceHistory::fetch_depth_price_history(
        &pool,
        &interval,
        start_time,
        &mongo_client  // Use the client directly before wrapping
    ).await {
        println!("Error fetching data: {}", e);
    }

    // Now wrap the client for the web server
    let app_state = web::Data::new(AppState {
        db: Mutex::new(Some(mongo_client)),
    });

    println!("Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .service(home_route)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}