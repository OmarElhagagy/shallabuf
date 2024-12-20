use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::DatabaseConnection, utils::internal_error};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeConnectionCreate {
    from_node_id: Uuid,
    to_node_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeConnection {
    id: Uuid,
    from_node_id: Uuid,
    to_node_id: Uuid,
}

pub async fn create(
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<PipelineNodeConnectionCreate>,
) -> Result<Json<PipelineNodeConnection>, StatusCode> {
    let connection = sqlx::query!(
        r#"
        INSERT INTO
            pipeline_nodes_connections (from_node_id, to_node_id)
        VALUES
            ($1, $2)
        RETURNING
            id, from_node_id, to_node_id
        "#,
        payload.from_node_id,
        payload.to_node_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineNodeConnection {
        id: connection.id,
        from_node_id: connection.from_node_id,
        to_node_id: connection.to_node_id,
    }))
}
