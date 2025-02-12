use chrono::{TimeZone, Utc};
pub mod services;
pub mod models;
use actix_web::{web::{self}, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;
use mongodb::Client;
mod routes;
mod middleware;

#[allow(dead_code)]
struct AppState {
    db: Mutex<Option<Client>>,
}
// pub const RUNEPOOL_START_TIME : i64= 1721865600; 
pub const RUNEPOOL_START_TIME : i64= 1648771200; 
const ONE_HOUR_SECS: u64 = 3_600;

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
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let mongo_client = services::db::Mongodb::connect_to_mongodb()
        .await
        .expect("Failed to connect to MongoDB");
    let db = services::db::Mongodb::new(mongo_client);
    let db_data = web::Data::new(db);

    let one_hour_ago = Utc::now().timestamp() - ONE_HOUR_SECS as i64;
    let _start_timer = one_hour_ago;
    
    let start_time = 1704326400;
    let _pool = String::from("BTC.BTC");
    let _interval = String::from("hour");
    println!("Starting fetch from: {}", format_timestamp(start_time));

    // if let Err(e) = services::fetch_earnings_history::fetch_earnings_history(
    //     // &pool,
    //     &interval,
    //     RUNEPOOL_START_TIME,
    //     &mongo_client  
    // ).await {
    //     println!("Error fetching data: {}", e);
    // }

    // let app_state = web::Data::new(AppState {
    //     db: Mutex::new(Some(db_data)),
    // });

    println!("Server starting at http://127.0.0.1:8080");

    HttpServer::new(move || {
        App::new().app_data(db_data.clone())
        
            .route("/health", web::get().to(health_check))
            .service(home_route)
            .service(routes::depth_history_routes::get_depth_history)
        //     .service(handlers::get_earnings)
        //    .service(handlers::get_earnings_pools)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}