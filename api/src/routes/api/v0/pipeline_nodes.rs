use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::{Coords, DatabaseConnection},
    utils::internal_error,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub id: Uuid,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub id: Uuid,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNode {
    pub id: Uuid,
    pub node_id: Uuid,
    pub node_version: String,
    pub trigger_id: Option<Uuid>,
    pub coords: serde_json::Value,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineConnection {
    pub id: Uuid,
    pub to_pipeline_node_input_id: Uuid,
    pub from_pipeline_node_output_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeCreationParams {
    pipeline_id: Uuid,
    node_id: Uuid,
    node_version: String,
    coords: Coords,
}

pub async fn create(
    DatabaseConnection(mut conn): DatabaseConnection,
    Json(payload): Json<PipelineNodeCreationParams>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let coords = serde_json::to_value(payload.coords.clone()).map_err(internal_error)?;

    let node = sqlx::query!(
        r#"
        INSERT INTO
            pipeline_nodes (pipeline_id, node_id, node_version, coords)
        VALUES
            ($1, $2, $3, $4)
        RETURNING id, node_id, node_version, trigger_id, coords
        "#,
        payload.pipeline_id,
        payload.node_id,
        payload.node_version,
        coords
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    let inputs = sqlx::query_as!(
        Input,
        r#"
        SELECT
            id, key
        FROM
            pipeline_node_inputs
        WHERE
            pipeline_node_id = $1
        "#,
        node.id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(internal_error)?;

    let outputs = sqlx::query_as!(
        Output,
        r#"
        SELECT
            id, key
        FROM
            pipeline_node_outputs
        WHERE
            pipeline_node_id = $1
        "#,
        node.id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineNode {
        id: node.id,
        node_id: node.node_id,
        node_version: node.node_version.clone(),
        trigger_id: node.trigger_id,
        coords: node.coords.clone(),
        inputs,
        outputs,
    }))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeUpdate {
    coords: Option<Coords>,
    trigger_id: Option<Uuid>,
}

pub async fn update(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<Uuid>,
    Json(params): Json<PipelineNodeUpdate>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let coords = params
        .coords
        .as_ref()
        .map(|c| serde_json::to_value(c).map_err(internal_error))
        .transpose()?;

    let node = sqlx::query!(
        r#"
        UPDATE
            pipeline_nodes
        SET
            coords = COALESCE($1, coords),
            trigger_id = COALESCE($2, trigger_id)
        WHERE
            id = $3
        RETURNING
            id, node_id, node_version, trigger_id, coords
        "#,
        coords,
        params.trigger_id,
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
        inputs: vec![],
        outputs: vec![],
    }))
}
