use app_state::{AppState, Broadcast, BroadcastEvent};
use async_nats::{self, jetstream};
use axum::{
    routing::{get, post},
    Router,
};
use db::seed::seed_database;
use dotenvy::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tokio::{io, sync::broadcast};
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

mod app_state;
mod extractors;
mod lib;
mod routes;
mod utils;

static JETSTREAM_NAME: &str = "PIPELINE_ACTIONS";

async fn run_migrations(pool: PgPool) {
    let rust_env = std::env::var("RUST_ENV").unwrap_or("dev".to_string());

    if rust_env == "dev" {
        if let Err(error) = sqlx::query!("DROP SCHEMA public CASCADE;")
            .execute(&pool)
            .await
        {
            error!("Failed to drop schema: {error:?}");
        } else if let Err(error) = sqlx::query!("CREATE SCHEMA public;").execute(&pool).await {
            error!("Failed to create schema: {error:?}");
        } else {
            info!("Schema dropped and recreated successfully");
        }
    }

    match db::MIGRATOR.run(&pool).await {
        Ok(()) => info!("Database migrated successfully"),
        Err(error) => error!("Failed to migrate database: {error:?}"),
    };

    if rust_env == "dev" {
        match seed_database(&pool).await {
            Ok(()) => info!("Database seeded successfully"),
            Err(error) => error!("Failed to seed database: {error:?}"),
        };
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let jetstream = jetstream::new(nats_client);

    jetstream
        .get_or_create_stream(jetstream::stream::Config {
            name: JETSTREAM_NAME.to_string(),
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            subjects: vec!["pipeline.>".to_string()],
            ..Default::default()
        })
        .await
        .expect("Failed to get or create JetStream");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .map_err(|error| {
            error!("Failed to connect to database: {error:?}");
            io::Error::new(io::ErrorKind::Other, "Failed to connect to database")
        })?;

    run_migrations(pg_pool.clone()).await;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).expect("Failed to create Redis client");
    let redis_connection_manager = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to create Redis connection manager");

    let (tx, _rx) = broadcast::channel::<BroadcastEvent>(100);

    let app_state = AppState {
        db: pg_pool,
        redis: redis_connection_manager,
        jetstream,
        broadcast: Broadcast(tx),
    };

    let app = Router::new()
        .route("/api/v0/auth/login", post(routes::api::v0::auth::login))
        .route("/api/v0/teams", get(routes::api::v0::teams::list))
        .route("/api/v0/pipelines", get(routes::api::v0::pipelines::list))
        .route(
            "/api/v0/pipelines",
            post(routes::api::v0::pipelines::create),
        )
        .route(
            "/api/v0/pipelines/:id",
            get(routes::api::v0::pipelines::details),
        )
        .route(
            "/api/v0/trigger/pipelines/:id",
            post(routes::api::v0::pipelines::trigger),
        )
        .route("/api/v0/nodes", get(routes::api::v0::nodes::list))
        .route(
            "/api/v0/pipeline_nodes",
            post(routes::api::v0::pipeline_nodes::create),
        )
        .route(
            "/api/v0/pipeline_nodes/:id",
            post(routes::api::v0::pipeline_nodes::update),
        )
        .route(
            "/api/v0/pipeline_node_connections",
            post(routes::api::v0::pipeline_node_connections::create),
        )
        .route("/api/v0/ws", get(routes::api::v0::events::ws_events))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
