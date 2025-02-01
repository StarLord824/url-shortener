mod models;
mod handlers;
mod utils;
mod error;

use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use redis::Client;
use crate::handlers::{shorten_url, redirect, qr_code, visual_hash};
use crate::error::ApiError;

#[actix_web::main]
async fn main() -> Result<(), ApiError> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Set up the PostgreSQL connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    // Set up the Redis client and connection
    let redis_client = Client::open(std::env::var("REDIS_URL")?)?;
    let redis_conn = redis_client.get_tokio_connection().await?;

    // Start the Actix-Web server
    HttpServer::new(move || {
        App::new()
            // Share the database pool with all handlers
            .app_data(web::Data::new(db_pool.clone()))
            // Share the Redis connection with all handlers
            .app_data(web::Data::new(redis_conn.clone()))
            // Register the URL shortening endpoint
            .service(
                web::resource("/shorten")
                    .route(web::post().to(shorten_url))
            )
            // Register the URL redirection endpoint
            .service(
                web::resource("/{id}")
                    .route(web::get().to(redirect))
            )
            // Register the QR code generation endpoint
            .service(
                web::resource("/qr/{id}")
                    .route(web::get().to(qr_code))
            )
            // Register the visual hash generation endpoint
            .service(
                web::resource("/visual/{id}")
                    .route(web::get().to(visual_hash))
            )
    })
    // Bind the server to the specified host and port
    .bind(format!(
        "{}:{}",
        std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
        std::env::var("PORT").unwrap_or_else(|_| "8080".into())
    ))?
    // Run the server
    .run()
    .await?;

    Ok(())
}