use app_state::{AppState, BroadcastEvent, ExecEventsConsumer, WsMessagesBroadcast};
use async_nats::{
    self,
    jetstream::{self},
};
use axum::{
    routing::{get, post, delete},
    Router,
};
use db::seed::seed_database;
use dotenvy::dotenv;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{env, process};
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
#[allow(clippy::too_many_lines)]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    let (loki_layer, loki_task) = tracing_loki::builder()
        .label("host", "mine")
        .expect("Failed to create Loki layer")
        .extra_field("pid", format!("{}", process::id()))
        .expect("Failed to add extra field to Loki layer")
        .build_url(
            env::var("LOKI_URL")
                .expect("LOKI_URL must be set")
                .parse()
                .expect("Failed to parse Loki URL"),
        )
        .expect("Failed to build Loki layer");

    tokio::spawn(loki_task);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(loki_layer)
        .init();

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let jetstream_actions = jetstream::new(nats_client.clone());

    jetstream_actions
        .get_or_create_stream(jetstream::stream::Config {
            name: JETSTREAM_NAME.to_string(),
            subjects: vec!["pipeline.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::WorkQueue,
            ..Default::default()
        })
        .await
        .expect("Failed to get or create JetStream");

    let jetstream_events = jetstream::new(nats_client.clone());

    let exec_events_consumer = ExecEventsConsumer(
        jetstream_events
            .get_stream(event_bridge::JETSTREAM_NAME.to_string())
            .await
            .expect("Failed to get or create JetStream")
            .create_consumer(async_nats::jetstream::consumer::pull::Config {
                durable_name: "EXEC_EVENTS_CONSUMER".to_string().into(),
                ack_policy: async_nats::jetstream::consumer::AckPolicy::Explicit,
                filter_subject: "exec.events".to_string(),
                ..Default::default()
            })
            .await
            .expect("Failed to create JetStream consumer"),
    );

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
    let ws_messages_broadcast = WsMessagesBroadcast(tx);

    let app_state = AppState {
        db: pg_pool,
        redis: redis_connection_manager,
        jetstream: jetstream_actions,
        ws_messages_broadcast,
        exec_events_consumer,
    };

    let api_v0 = Router::new()
        .route("/auth/login", post(routes::api::v0::auth::login))
        .route("/auth/logout", post(routes::api::v0::auth::logout))
        .route("/auth/session", get(routes::api::v0::auth::session))
        .route("/teams", get(routes::api::v0::teams::list))
        .route("/pipelines", get(routes::api::v0::pipelines::list))
        .route("/pipelines", post(routes::api::v0::pipelines::create))
        .route("/pipelines/:id", get(routes::api::v0::pipelines::details))
        .route(
            "/trigger/pipelines/:id",
            post(routes::api::v0::pipelines::trigger),
        )
        .route(
            "/pipeline-triggers/:id",
            post(routes::api::v0::pipeline_triggers::update),
        )
        .route("/nodes", get(routes::api::v0::nodes::list))
        .route(
            "/pipeline-nodes",
            post(routes::api::v0::pipeline_nodes::create),
        )
        .route(
            "/pipeline-nodes/:id",
            post(routes::api::v0::pipeline_nodes::update),
        )
        .route(
            "/pipeline-nodes/:id",
            delete(routes::api::v0::pipeline_nodes::delete)
        )
        .route(
            "/pipeline-node-connections",
            post(routes::api::v0::pipeline_node_connections::create),
        )
        .route(
            "/pipeline-execs/:id",
            get(routes::api::v0::pipeline_execs::subscribe),
        )
        .route("/ws", get(routes::api::v0::events::ws_events));

    let app = Router::new()
        .nest("/api/v0", api_v0)
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
