use axum::Json;
use hyper::StatusCode;
use uuid::Uuid;

use crate::{app_state::DatabaseConnection, utils::internal_error};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

pub async fn list(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<Node>>, StatusCode> {
    let nodes = sqlx::query!(
        r#"
        SELECT
            id, name, description
        FROM
            nodes
        "#,
    )
    .fetch_all(&mut *conn)
    .await
    .map(|rows| {
        rows.iter()
            .map(|row| Node {
                id: row.id,
                name: row.name.clone(),
                description: row.description.clone(),
            })
            .collect::<Vec<Node>>()
    })
    .map_err(internal_error)?;

    Ok(Json(nodes))
}
