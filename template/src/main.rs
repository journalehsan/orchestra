use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod controllers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::AppConfig::from_env();

    // create_if_missing(true) automatically creates database.db on first run
    let connect_opts = SqliteConnectOptions::from_str(&cfg.database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_opts)
        .await?;

    // Run all pending migrations from the ./migrations directory
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database ready");

    tracing::info!("Starting server at http://{}:{}", cfg.host, cfg.port);

    let pool = web::Data::new(pool);
    let host = cfg.host.clone();
    let port = cfg.port;

    HttpServer::new(move || {
        App::new()
            .app_data(pool.clone())
            .configure(routes::configure)
    })
    .bind((host.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
