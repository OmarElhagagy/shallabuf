use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app_state::DatabaseConnection, utils::internal_error};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeConnectionCreate {
    to_pipeline_node_input_id: Uuid,
    from_pipeline_node_output_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeConnection {
    id: Uuid,
    to_pipeline_node_input_id: Uuid,
    from_pipeline_node_output_id: Uuid,
}

pub async fn create(
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<PipelineNodeConnectionCreate>,
) -> Result<Json<PipelineNodeConnection>, StatusCode> {
    let connection = sqlx::query!(
        r#"
        INSERT INTO
            pipeline_node_connections (to_pipeline_node_input_id, from_pipeline_node_output_id)
        VALUES
            ($1, $2)
        RETURNING
            id, to_pipeline_node_input_id, from_pipeline_node_output_id
        "#,
        payload.to_pipeline_node_input_id,
        payload.from_pipeline_node_output_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineNodeConnection {
        id: connection.id,
        to_pipeline_node_input_id: connection.to_pipeline_node_input_id,
        from_pipeline_node_output_id: connection.from_pipeline_node_output_id,
    }))
}
