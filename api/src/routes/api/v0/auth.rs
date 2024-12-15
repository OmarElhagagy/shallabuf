use crate::app_state::{DatabaseConnection, RedisConnection};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use axum::Json;
use base32;
use chrono::Utc;
use db::dtos::KeyProviderType;
use hex;
use hyper::StatusCode;
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::error;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub token: String,
    pub user_id: Uuid,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

fn generate_session_token() -> String {
    let mut bytes = [0u8; 20];
    rand::thread_rng().fill(&mut bytes);
    base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes).to_lowercase()
}

fn generate_session_id(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize()).to_lowercase()
}

fn to_redis_session_key(session_id: &str) -> String {
    format!("session:{session_id}")
}

async fn create_session(
    mut redis: redis::aio::ConnectionManager,
    token: &str,
    user_id: Uuid,
) -> Result<Session, StatusCode> {
    let session_id = generate_session_id(token);

    let expires_at = chrono::Utc::now() + chrono::Duration::minutes(30);
    let session = Session {
        token: session_id.clone(),
        user_id,
        expires_at,
    };

    let Ok(session_str) = serde_json::to_string(&session) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let _: () = redis
        .set_ex(
            format!("session:{}", session.token),
            session_str,
            session.expires_at.timestamp() as u64,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(session)
}

async fn validate_session_token(
    mut redis: redis::aio::ConnectionManager,
    token: &str,
) -> Result<Option<Session>, StatusCode> {
    let session_id = generate_session_id(token);

    let session_str: String = match redis
        .get(to_redis_session_key(&session_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        Some(session_str) => session_str,
        None => {
            return Ok(None);
        }
    };

    let mut session = serde_json::from_str::<Session>(&session_str)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = Utc::now();

    if now.timestamp() >= session.expires_at.timestamp() {
        // Delete expired session, not sure if this is necessary bc we're setting an expiration time for Redis
        let _: () = redis
            .del(to_redis_session_key(&session_id))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(None);
    }

    let fifteen_minutes = chrono::Duration::minutes(15);

    if now >= session.expires_at - fifteen_minutes {
        // Extend session by 15 minutes
        session.expires_at = Utc::now() + fifteen_minutes;

        let updated_session =
            serde_json::to_string(&session).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let _: () = redis
            .set_ex(
                to_redis_session_key(&session_id),
                updated_session,
                session.expires_at.timestamp() as u64,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Some(session))
}

async fn invalidate_session(
    mut redis: redis::aio::ConnectionManager,
    session_id: &str,
) -> Result<(), StatusCode> {
    let _: () = redis
        .del(to_redis_session_key(session_id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub session: Session,
}

pub async fn login(
    DatabaseConnection(mut conn): DatabaseConnection,
    RedisConnection(redis): RedisConnection,
    Json(LoginRequest { email, password }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query!(
        r#"
        SELECT
            users.id, users.password_hash
        FROM
            users
        LEFT JOIN
            keys ON keys.user_id = users.id
        WHERE
            users.email = $1
        AND
            keys.provider = $2
        "#,
        email,
        KeyProviderType::Password as KeyProviderType,
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(internal_error)?;

    let Some(user) = user else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let Some(password_hash) = user.password_hash else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let parsed_password_hash = PasswordHash::new(&password_hash).map_err(internal_error)?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_password_hash)
        .map_err(internal_error)?;

    let token = generate_session_token();
    let session = create_session(redis, &token, user.id).await?;

    Ok(Json(LoginResponse { session }))
}

fn internal_error<T: std::fmt::Debug>(error: T) -> StatusCode {
    error!("Internal error: {error:?}");
    StatusCode::INTERNAL_SERVER_ERROR
}
