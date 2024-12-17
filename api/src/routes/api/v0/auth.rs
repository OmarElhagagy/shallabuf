use crate::{
    app_state::{DatabaseConnection, RedisConnection},
    lib::session::{create_session, generate_session_token},
    utils::internal_error,
};
use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
use axum::Json;
use db::dtos::KeyProviderType;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub async fn login(
    DatabaseConnection(mut conn): DatabaseConnection,
    RedisConnection(redis): RedisConnection,
    Json(LoginRequest { email, password }): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query!(
        r#"
        SELECT
            users.id, users.name, users.password_hash
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
    let session = create_session(redis, &token, user.id, &user.name).await?;

    Ok(Json(LoginResponse {
        token,
        expires_at: session.expires_at,
    }))
}
