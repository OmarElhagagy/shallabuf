use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
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

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError {
            error: err.to_string(),
        }
    }
}

pub async fn delete(
    DatabaseConnection(mut conn): DatabaseConnection,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    info!("Attempting to delete pipeline node: {}", id);

    let mut tx = conn
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    // Check if node exists
    let node_exists = sqlx::query!("SELECT id FROM pipeline_nodes WHERE id = $1", id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?
        .is_some();

    if !node_exists {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ApiError {
                error: "Pipeline node not found".to_string(),
            }),
        ));
    }

    // Delete connections
    sqlx::query!(
        r#"
        DELETE FROM pipeline_node_connections
        WHERE source_node_id = $1 OR target_node_id = $1
        "#,
        id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    // Delete inputs and outputs
    sqlx::query!("DELETE FROM pipeline_node_inputs WHERE pipeline_node_id = $1", id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    sqlx::query!("DELETE FROM pipeline_node_outputs WHERE pipeline_node_id = $1", id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    // Delete the node
    sqlx::query!("DELETE FROM pipeline_nodes WHERE id = $1", id)
        .execute(&mut *tx)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    tx.commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError::from(e))))?;

    info!("Successfully deleted pipeline node: {}", id);
    Ok(StatusCode::NO_CONTENT)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_delete_pipeline_node_success() {
        // Replace with your test database URL
        let pool = PgPool::connect("postgres://user:pass@localhost/test_db")
            .await
            .unwrap();
        let mut conn = DatabaseConnection(pool.acquire().await.unwrap());

        // Insert test data
        let pipeline_id = Uuid::new_v4();
        let node_id = Uuid::new_v4();
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO pipeline_nodes (id, pipeline_id, node_id, node_version, coords) VALUES ($1, $2, $3, $4, $5)",
            id,
            pipeline_id,
            node_id,
            "1.0",
            serde_json::json!({"x": 0, "y": 0})
        )
        .execute(&mut *conn.0)
        .await
        .unwrap();

        // Delete the node
        let result = delete(conn, Path(id)).await;
        assert!(matches!(result, Ok(StatusCode::NO_CONTENT)));

        // Verify deletion
        let exists = sqlx::query!("SELECT id FROM pipeline_nodes WHERE id = $1", id)
            .fetch_optional(&mut *pool.acquire().await.unwrap())
            .await
            .unwrap()
            .is_none();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_delete_pipeline_node_not_found() {
        let pool = PgPool::connect("postgres://user:pass@localhost/test_db")
            .await
            .unwrap();
        let conn = DatabaseConnection(pool.acquire().await.unwrap());

        let id = Uuid::new_v4();
        let result = delete(conn, Path(id)).await;
        assert!(matches!(
            result,
            Err((StatusCode::NOT_FOUND, _))
        ));
    }
}
