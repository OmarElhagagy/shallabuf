use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::{Coords, DatabaseConnection},
    utils::internal_error,
    PipelineNode,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeCreate {
    pipeline_id: Uuid,
    node_id: Uuid,
    node_version: String,
    coords: Coords,
}

pub async fn create(
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<PipelineNodeCreate>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let coords = serde_json::to_value(payload.coords.clone()).map_err(internal_error)?;

    let node = sqlx::query!(
        r#"
        INSERT INTO
            pipeline_nodes (pipeline_id, node_id, node_version, coords)
        VALUES
            ($1, $2, $3, $4)
        RETURNING
            id, node_id, node_version, trigger_id, coords
        "#,
        payload.pipeline_id,
        payload.node_id,
        payload.node_version,
        coords
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineNode {
        id: node.id,
        node_id: node.node_id,
        node_version: node.node_version,
        trigger_id: node.trigger_id,
        coords: node.coords,
    }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeUpdate {
    coords: Coords,
}

pub async fn update(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<Uuid>,
    Json(payload): Json<PipelineNodeUpdate>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let coords = serde_json::to_value(payload.coords.clone()).map_err(internal_error)?;

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
        coords,
        id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineNode {
        id: node.id,
        node_id: node.node_id,
        node_version: node.node_version,
        trigger_id: node.trigger_id,
        coords: node.coords,
    }))
}
