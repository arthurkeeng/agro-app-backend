use actix_web::{web, App, HttpServer, middleware::Logger, Result as ActixResult};
use actix_cors::Cors;
use std::env;
use database::Database;

use crate::handlers::farmers::{register_farmer, verify_phone};

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


    HttpServer::new(move||{
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .app_data(web::Data::new(db.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/api/health_check", web::get().to(health_check))
            .service(
                web::scope("/api/farmers")
                .route("register", web::post().to(register_farmer))
                .route("verify-phone", web::post().to(verify_phone))
            )

    })
    .bind("127.0.0.1:3001")?
    .run()
    .await
    
}

async fn health_check() -> ActixResult<&'static str>{
    println!("The health check endpoint was called");
    Ok("Ok")
}


