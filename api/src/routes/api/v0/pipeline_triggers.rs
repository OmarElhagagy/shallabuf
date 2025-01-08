use axum::{extract::Path, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::{Coords, DatabaseConnection},
    extractors::session::Session,
    utils::internal_error,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePipelineTriggerParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coords: Option<Coords>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTrigger {
    pub id: Uuid,
    pub coords: serde_json::Value,
    pub config: serde_json::Value,
}

pub async fn update(
    DatabaseConnection(mut conn): DatabaseConnection,
    Session(_): Session,
    Path(id): Path<Uuid>,
    Json(params): Json<UpdatePipelineTriggerParams>,
) -> Result<Json<PipelineTrigger>, StatusCode> {
    let coords = params
        .coords
        .as_ref()
        .map(|c| serde_json::to_value(c).map_err(internal_error))
        .transpose()?;

    let trigger = sqlx::query!(
        r#"
        UPDATE
            pipeline_triggers
        SET
            coords = COALESCE($1, coords)
        WHERE
            id = $2
        RETURNING
            id, coords, config
        "#,
        coords,
        id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineTrigger {
        id: trigger.id,
        coords: trigger.coords,
        config: trigger.config,
    }))
}
