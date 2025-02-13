#![recursion_limit = "256"]
use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use mongodb::Client;
use std::env;
use tokio::sync::Mutex;
pub mod database;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

#[allow(dead_code)]
struct AppState {
    db: Mutex<Option<Client>>,
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
    let mongo_client = database::db::Mongodb::connect_to_mongodb()
        .await
        .expect("Failed to connect to MongoDB");

    // actix_web::rt::spawn(services::scheduler::start_hourly_data_fetch(
    //     mongo_client.clone(),
    // ));

    let db = database::db::Mongodb::new(mongo_client);
    let db_data = web::Data::new(db);

    println!(
        "Server starting at http://127.0.0.1:{}",
        env::var("PORT").unwrap_or("8080".to_string())
    );

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .route("/health", web::get().to(health_check))
            .service(home_route)
            .service(routes::depth_history_routes::get_depth_history)
            .service(routes::swaps_history_routes::get_swaps_history)
            .service(routes::rune_pool_history_route::get_runepool_history)
            .service(routes::earning_history_route::get_earnings_history)
    })
    .bind(format!("127.0.0.1:{}", env::var("PORT").unwrap()))?
    .run()
    .await
}
