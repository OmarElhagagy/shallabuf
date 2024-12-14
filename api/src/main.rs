use app_state::{AppState, Broadcast, DatabaseConnection, JetStream, RedisConnection, WsAction};
use async_nats::{self, jetstream};
use axum::{
    extract::{
        ws::{self, Message, WebSocket, WebSocketUpgrade},
        Extension, Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Error, Json, Router,
};
use db::dtos::PipelineExecPayloadParams;
use db::seed::seed_database;
use dotenvy::dotenv;
use redis::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{
    pool::PoolConnection,
    postgres::{PgListener, PgPool, PgPoolOptions},
    FromRow, Postgres,
};
use std::net::SocketAddr;
use std::{collections::HashSet, sync::Arc};
use tokio::{
    io,
    sync::{broadcast, Mutex},
};
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};
use uuid::Uuid;

mod app_state;

static JETSTREAM_NAME: &str = "PIPELINE_ACTIONS";

#[derive(Debug, Serialize, Deserialize)]
struct PipelineNode {
    id: Uuid,
    node_id: Uuid,
    node_version: String,
    trigger_id: Option<Uuid>,
    coords: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PipelineNodeUpdate {
    coords: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct PipelineConnection {
    id: Uuid,
    from_node_id: Uuid,
    to_node_id: Uuid,
}

#[derive(Debug, Serialize)]
struct Pipeline {
    id: Uuid,
    name: String,
    description: Option<String>,
    nodes: Vec<PipelineNode>,
    connections: Vec<PipelineConnection>,
}

async fn pipelines(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<Pipeline>>, StatusCode> {
    let pipelines = sqlx::query!(
        r#"
        SELECT
            id, name, description
        FROM
            pipelines
        "#
    )
    .fetch_all(&mut *conn)
    .await
    .map(|rows| {
        rows.iter()
            .map(|row| Pipeline {
                id: row.id,
                name: row.name.clone(),
                description: row.description.clone(),
                nodes: vec![],
                connections: vec![],
            })
            .collect::<Vec<Pipeline>>()
    })
    .map_err(|error| {
        error!("Database error: {error:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(pipelines))
}

async fn pipeline_details(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<Uuid>,
) -> Result<Json<Pipeline>, StatusCode> {
    let pipeline = sqlx::query!(
        r#"
        SELECT
            p.id AS pipeline_id, p.name, p.description,
            pn.id AS node_id, pn.node_id AS node_node_id, pn.node_version, pn.trigger_id, pn.coords,
            pc.id AS connection_id, pc.from_node_id, pc.to_node_id
        FROM
            pipelines p
        LEFT JOIN
            pipeline_nodes pn ON pn.pipeline_id = p.id
        LEFT JOIN
            pipeline_nodes_connections pc ON pc.from_node_id = pn.id OR pc.to_node_id = pn.id
        WHERE
            p.id = $1
        "#,
        id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|error| {
        error!("Database error: {error:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if pipeline.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut seen_nodes = HashSet::new();
    let nodes = pipeline
        .iter()
        .filter_map(|row| {
            if seen_nodes.insert(row.node_id) {
                Some(PipelineNode {
                    id: row.node_id,
                    node_id: row.node_node_id,
                    node_version: row.node_version.clone(),
                    trigger_id: row.trigger_id,
                    coords: row.coords.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    let mut seen_connections = HashSet::new();
    let connections = pipeline
        .iter()
        .filter_map(|row| {
            if seen_connections.insert(row.connection_id) {
                Some(PipelineConnection {
                    id: row.connection_id,
                    from_node_id: row.from_node_id,
                    to_node_id: row.to_node_id,
                })
            } else {
                None
            }
        })
        .collect();

    let pipeline = Pipeline {
        id: pipeline[0].pipeline_id,
        name: pipeline[0].name.clone(),
        description: pipeline[0].description.clone(),
        nodes,
        connections,
    };

    Ok(Json(pipeline))
}

async fn update_pipeline_node(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<Uuid>,
    Json(node): Json<PipelineNodeUpdate>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let node = sqlx::query!(
        r#"
        UPDATE
            pipeline_nodes
        SET
            coords = COALESCE($1, coords)
        WHERE
            id = $2
        RETURNING
            id, node_id, node_version, trigger_id, coords
        "#,
        node.coords,
        id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|error| {
        error!("Database error: {error:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(PipelineNode {
        id: node.id,
        node_id: node.node_id,
        node_version: node.node_version,
        trigger_id: node.trigger_id,
        coords: node.coords,
    }))
}

#[derive(Serialize)]
struct PipelineTriggerResponse {
    pipeline_exec_id: Uuid,
}

async fn trigger_pipeline(
    Path(id): Path<String>,
    DatabaseConnection(mut conn): DatabaseConnection,
    State(stream): State<JetStream>,
    Json(params): Json<PipelineExecPayloadParams>,
) -> Result<Json<PipelineTriggerResponse>, StatusCode> {
    info!("Received request to trigger pipeline with id: {id}");
    let pipeline_id = Uuid::parse_str(&id).map_err(|_| {
        warn!("Invalid UUID format for pipeline id: {id}");
        StatusCode::BAD_REQUEST
    })?;

    // let pipeline_exec_id = sqlx::query_as(
    //     r"
    //     INSERT INTO pipeline_exec (pipeline_id)
    //     VALUES ($1)
    //     RETURNING id
    //     ",
    //     pipeline_id,
    // );

    // let pipeline_exec = pipeline_exec::Entity::insert(pipeline_exec::ActiveModel {
    //     pipeline_id: Set(pipeline_id),
    //     ..Default::default()
    // })
    // .exec(&pool)
    // .await
    // .map_err(|error| {
    //     error!("Database error: {error:?}");
    //     StatusCode::INTERNAL_SERVER_ERROR
    // })?;

    // info!(
    //     "Pipeline execution record created with id: {}",
    //     pipeline_exec.last_insert_id
    // );

    // let payload_bytes = serde_json::to_string(&dtos::PipelineExecPayload {
    //     pipeline_id,
    //     pipeline_exec_id: pipeline_exec.last_insert_id,
    //     params,
    // })
    // .map_err(|error| {
    //     error!("Failed to serialize payload: {error:?}");
    //     StatusCode::INTERNAL_SERVER_ERROR
    // })?
    // .into();

    // if let Err(error) = stream.publish("pipeline.exec", payload_bytes).await {
    //     error!("Failed to publish message to JetStream: {error:?}");
    // } else {
    //     debug!(
    //         "Published message to JetStream for pipeline_exec_id: {}",
    //         pipeline_exec.last_insert_id
    //     );
    // }

    // Ok(Json(PipelineTriggerResponse {
    //     pipeline_exec_id: pipeline_exec.last_insert_id
    // }))

    Ok(Json(PipelineTriggerResponse {
        pipeline_exec_id: Uuid::new_v4(),
    }))
}

async fn ws_events(
    ws: WebSocketUpgrade,
    DatabaseConnection(conn): DatabaseConnection,
    RedisConnection(redis): RedisConnection,
    State(sender): State<Broadcast>,
) -> impl IntoResponse {
    let sender_id = Uuid::new_v4(); // Assign unique ID for this connection
    ws.on_upgrade(move |socket| handle_ws_events(socket, conn, redis, sender.0, sender_id))
}

async fn handle_ws_events(
    mut socket: WebSocket,
    conn: PoolConnection<Postgres>,
    redis: Client,
    sender: broadcast::Sender<WsAction>,
    sender_id: Uuid,
) {
    let mut receiver = sender.subscribe();

    loop {
        tokio::select! {
            res = socket.recv() => {
                match res {
                    Some(Ok(ws::Message::Text(message))) => {
                        if let Ok(mut message) = serde_json::from_str::<WsAction>(&message) {
                            match message {
                                WsAction::UpdateNode(ref mut update) => {
                                    update.payload.sender_id = Some(sender_id);

                                    if let Err(error) = sender.send(WsAction::UpdateNode(update.clone())) {
                                        error!("Failed to broadcast message: {error:?}");
                                    } else {
                                        debug!("Broadcasted message: {update:?}");
                                    }
                                }
                                WsAction::AddEditorParticipant(ref update) => {
                                    let value = json!({
                                        "action": "add_editor_participant",
                                        "payload": {
                                            "pipeline_id": update.pipeline_id,
                                            "user_id": update.user_id,
                                            "name": update.name,
                                        },
                                    });

                                    if let Ok(text) = serde_json::to_string(&value) {
                                        if let Err(error) = socket.send(Message::Text(text)).await {
                                            error!("Failed to send message to WebSocket: {error:?}");
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => tracing::debug!("client disconnected abruptly: {error}"),
                    None => break,
                }
            },
            res = receiver.recv() => {
                match res {
                    Ok(action) => {
                        match action {
                            WsAction::UpdateNode(ref update) => {
                                if let Some(update_sender_id) = update.payload.sender_id {
                                    if update_sender_id == sender_id {
                                        continue;
                                    }
                                }

                                let value = json!({
                                    "action": "update_node",
                                    "payload": {
                                        "id": update.payload.id,
                                        "coords": update.payload.coords,
                                        "sender_id": update.payload.sender_id,
                                    },
                                });

                                if let Ok(text) = serde_json::to_string(&value) {
                                    if let Err(error) = socket.send(Message::Text(text)).await {
                                        error!("Failed to send message to WebSocket: {error:?}");
                                        break;
                                    }
                                }
                            }
                            WsAction::AddEditorParticipant(ref update) => {
                                let value = json!({
                                    "action": "add_editor_participant",
                                    "payload": {
                                        "pipeline_id": update.pipeline_id,
                                        "user_id": update.user_id,
                                        "name": update.name,
                                    },
                                });

                                if let Ok(text) = serde_json::to_string(&value) {
                                    if let Err(error) = socket.send(Message::Text(text)).await {
                                        error!("Failed to send message to WebSocket: {error:?}");
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(error) => {
                        error!("Failed to receive broadcast message: {error:?}");
                        break;
                    }
                }
            }
        }
    }
}

async fn run_migrations(pool: PgPool) {
    let rust_env = std::env::var("RUST_ENV").unwrap_or("dev".to_string());

    if (rust_env == "dev") {
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
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    run_migrations(pool.clone()).await;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = Client::open(redis_url).expect("Failed to create Redis client");

    let (tx, _rx) = broadcast::channel::<WsAction>(100);

    let app_state = AppState {
        db: pool,
        jetstream: JetStream(jetstream),
        broadcast: Broadcast(tx),
        redis: redis_client,
    };

    let app = Router::new()
        .route("/api/v0/pipelines", get(pipelines))
        .route("/api/v0/pipelines/:id", get(pipeline_details))
        .route("/api/v0/trigger/pipelines/:id", post(trigger_pipeline))
        .route("/api/v0/pipeline_nodes/:id", post(update_pipeline_node))
        .route("/api/v0/ws", get(ws_events))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
