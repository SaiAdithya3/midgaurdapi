mod services {
    pub mod db;
}

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use std::sync::Mutex;
use mongodb::Client;

// Wrap MongoDB client in app state
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
    let mongo_client = services::db::Mongodb::connect_to_mongodb()
        .await
        .expect("Failed to connect to MongoDB");

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