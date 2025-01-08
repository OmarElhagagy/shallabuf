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

    let nodes = sqlx::query!(
        r#"
        WITH inserted_pipeline_node AS (
            INSERT INTO
                pipeline_nodes (pipeline_id, node_id, node_version, coords)
            VALUES
                ($1, $2, $3, $4)
            RETURNING
                id, node_id, node_version, trigger_id, coords
        )
        SELECT
            pn.id, pn.node_id, pn.node_version, pn.trigger_id, pn.coords,
            pni.id AS "input_id?", pni.key as "input_key?", pno.id AS "output_id?", pno.key AS "output_key?"
        FROM
            inserted_pipeline_node pn
        LEFT JOIN
            pipeline_node_inputs pni ON pni.pipeline_node_id = pn.id
        LEFT JOIN
            pipeline_node_outputs pno ON pno.pipeline_node_id = pn.id
        "#,
        payload.pipeline_id,
        payload.node_id,
        payload.node_version,
        coords
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(internal_error)?;

    let inputs = nodes
        .iter()
        .filter_map(|row| {
            if let (Some(id), Some(key)) = (row.input_id, &row.input_key) {
                Some(Input {
                    id,
                    key: key.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<Input>>();

    let outputs = nodes
        .iter()
        .filter_map(|row| {
            if let (Some(id), Some(key)) = (row.output_id, &row.output_key) {
                Some(Output {
                    id,
                    key: key.clone(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<Output>>();

    let node = nodes.first().ok_or(StatusCode::NOT_FOUND)?;

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
    Json(payload): Json<PipelineNodeUpdate>,
) -> Result<Json<PipelineNode>, StatusCode> {
    let coords = payload
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
        payload.trigger_id,
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
