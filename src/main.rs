use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer, Result as ActixResult};
use actix_cors::Cors;
use std::env;
use database::Database;

use crate::handlers::farmers::{farmer_login, register_farmer, verify_phone};

mod models;
mod handlers;
mod database;
mod services;
mod utils;
mod errors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::new(&database_url).await.expect("Failed to connect to the database");

    // Get the port from the environment variable (Render provides this)
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3001".to_string()) // default for local development
        .parse::<u16>()
        .expect("PORT must be a number");

    println!("Starting server on port: {}", port);
    let secret_key = Key::generate();
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(), secret_key.clone()
            ))
            .wrap(Logger::default())
            .route("/api/health_check", web::get().to(health_check))
            .service(
                web::scope("/api/farmers")
                    .route("/register", web::post().to(register_farmer))
                    .route("/login", web::post().to(farmer_login))
                    .route("/verify-phone", web::post().to(verify_phone)),
            )
            .service(
                web::scope("/api/products")
                .route("" , web::post().to(handlers::products::add_products))
            )
    })
    .bind(("0.0.0.0", port))? // Bind to all interfaces
    .run()
    .await
}

async fn health_check() -> ActixResult<&'static str> {
    println!("The health check endpoint was called");
    Ok("Ok")
}
