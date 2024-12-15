use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::Pool<sqlx::Postgres>,
    pub jetstream: JetStream,
    pub broadcast: Broadcast,
    pub redis: redis::aio::ConnectionManager,
}

pub struct DatabaseConnection(pub sqlx::pool::PoolConnection<sqlx::Postgres>);

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let conn = state.db.acquire().await.map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to acquire connection: {error}"),
            )
        })?;

        Ok(Self(conn))
    }
}

#[derive(Clone)]
pub struct JetStream(pub async_nats::jetstream::Context);

impl FromRef<AppState> for JetStream {
    fn from_ref(state: &AppState) -> Self {
        Self(state.jetstream.0.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineNodeUpdate {
    pub node_id: Uuid,
    pub coords: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddEditorParticipant {
    pub pipeline_id: String,
    pub user_id: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoveEditorParticipant {
    pub pipeline_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsActionPayload<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<Uuid>,
    pub payload: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "action")]
pub enum WsAction {
    AddEditorParticipant(WsActionPayload<AddEditorParticipant>),
    RemoveEditorParticipant(WsActionPayload<RemoveEditorParticipant>),
    UpdateNode(WsActionPayload<PipelineNodeUpdate>),
}

#[derive(Clone)]
pub struct Broadcast(pub broadcast::Sender<WsAction>);

impl FromRef<AppState> for Broadcast {
    fn from_ref(state: &AppState) -> Self {
        Self(state.broadcast.0.clone())
    }
}

pub struct RedisConnection(pub redis::aio::ConnectionManager);

#[async_trait]
impl<S> FromRequestParts<S> for RedisConnection
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        Ok(Self(state.redis.clone()))
    }
}
