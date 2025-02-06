use chrono::Utc;
use hyper::StatusCode;
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub username: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub fn generate_session_token() -> String {
    let mut bytes = [0u8; 20];
    rand::rng().fill(&mut bytes);
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes).to_lowercase()
}

pub fn generate_session_id(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize()).to_lowercase()
}

pub fn to_redis_session_key(session_id: &str) -> String {
    format!("session:{session_id}")
}

pub async fn create_session(
    mut redis: redis::aio::ConnectionManager,
    token: &str,
    user_id: Uuid,
    username: &str,
) -> Result<Session, StatusCode> {
    let session_id = generate_session_id(token);

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
    let session = Session {
        id: session_id,
        user_id,
        username: username.to_string(),
        expires_at,
    };

    let Ok(session_str) = serde_json::to_string(&session) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let _: () = redis
        .set_ex(
            format!("session:{}", session.id),
            session_str,
            session.expires_at.timestamp() as u64,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(session)
}

pub async fn validate_session_token(
    mut redis: redis::aio::ConnectionManager,
    token: &str,
) -> Result<Option<Session>, StatusCode> {
    info!("Validating session token: {token}");
    let session_id = generate_session_id(token);

    let session_str: String = if let Some(session_str) = redis
        .get(to_redis_session_key(&session_id))
        .await
        .map_err(|error| {
            error!("Redis GET failed for session_id: {session_id}, error: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })? {
        info!("Session found for session_id: {session_id}");
        session_str
    } else {
        warn!("No session found for session_id: {session_id}");
        return Ok(None);
    };

    let mut session = serde_json::from_str::<Session>(&session_str).map_err(|error| {
        error!("Failed to parse session JSON: {error}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let now = Utc::now();

    if now.timestamp() >= session.expires_at.timestamp() {
        info!("Session expired for session_id: {session_id}");
        let _: () = redis
            .del(to_redis_session_key(&session_id))
            .await
            .map_err(|error| {
                error!("Failed to delete expired session_id: {session_id}, error: {error}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        return Ok(None);
    }

    let fifteen_minutes = chrono::Duration::minutes(15);

    if now >= session.expires_at - fifteen_minutes {
        info!("Extending session for session_id: {}", session_id);
        session.expires_at = Utc::now() + fifteen_minutes;

        let updated_session = serde_json::to_string(&session).map_err(|error| {
            error!("Failed to serialize updated session: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let _: () = redis
            .set_ex(
                to_redis_session_key(&session_id),
                updated_session,
                session.expires_at.timestamp() as u64,
            )
            .await
            .map_err(|error| {
                error!("Failed to update session in Redis for session_id: {session_id}, error: {error}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    Ok(Some(session))
}

pub async fn invalidate_session(
    mut redis: redis::aio::ConnectionManager,
    session_id: &str,
) -> Result<(), StatusCode> {
    let _: () = redis
        .del(to_redis_session_key(session_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}
