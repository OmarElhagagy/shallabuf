use axum::Json;
use hyper::StatusCode;
use serde::Serialize;
use uuid::Uuid;

use crate::{app_state::DatabaseConnection, extractors::session::Session, utils::internal_error};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamListItem {
    pub id: Uuid,
    pub name: String,
}

pub async fn list(
    DatabaseConnection(mut conn): DatabaseConnection,
    Session(session): Session,
) -> Result<Json<Vec<TeamListItem>>, StatusCode> {
    let teams = sqlx::query!(
        r#"
            SELECT
                teams.id, teams.name
            FROM
                user_teams
            JOIN
                teams ON user_teams.team_id = teams.id
            WHERE
                user_teams.user_id = $1
        "#,
        session.user_id
    )
    .fetch_all(&mut *conn)
    .await
    .map(|rows| {
        rows.iter()
            .map(|row| TeamListItem {
                id: row.id,
                name: row.name.clone(),
            })
            .collect::<Vec<TeamListItem>>()
    })
    .map_err(internal_error)?;

    if teams.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(teams))
}
