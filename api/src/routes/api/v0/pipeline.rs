use std::collections::HashSet;

use axum::{extract::Path, Json};
use hyper::StatusCode;
use redis::AsyncCommands;
use tracing::error;
use uuid::Uuid;

use crate::{
    app_state::{DatabaseConnection, RedisConnection},
    Pipeline, PipelineConnection, PipelineNode, PipelineParticipant,
};

pub async fn details(
    DatabaseConnection(mut conn): DatabaseConnection,
    RedisConnection(mut redis): RedisConnection,
    Path(id): Path<Uuid>,
) -> Result<Json<Pipeline>, StatusCode> {
    let pipeline = sqlx::query!(
        r#"
        SELECT
            p.id AS pipeline_id, p.name, p.description,
            pn.id AS node_id, pn.node_id AS node_node_id, pn.node_version, pn.trigger_id, pn.coords,
            pc.id AS connection_id, pc.from_node_id, pc.to_node_id
        FROM
            pipelines p
        LEFT JOIN
            pipeline_nodes pn ON pn.pipeline_id = p.id
        LEFT JOIN
            pipeline_nodes_connections pc ON pc.from_node_id = pn.id OR pc.to_node_id = pn.id
        WHERE
            p.id = $1
        "#,
        id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|error| {
        error!("Database error: {error:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if pipeline.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let mut seen_nodes = HashSet::new();
    let nodes = pipeline
        .iter()
        .filter_map(|row| {
            if seen_nodes.insert(row.node_id) {
                Some(PipelineNode {
                    id: row.node_id,
                    node_id: row.node_node_id,
                    node_version: row.node_version.clone(),
                    trigger_id: row.trigger_id,
                    coords: row.coords.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    let mut seen_connections = HashSet::new();
    let connections = pipeline
        .iter()
        .filter_map(|row| {
            if seen_connections.insert(row.connection_id) {
                Some(PipelineConnection {
                    id: row.connection_id,
                    from_node_id: row.from_node_id,
                    to_node_id: row.to_node_id,
                })
            } else {
                None
            }
        })
        .collect();

    let participants: Vec<(String, String)> = redis
        .hgetall("pipeline:1:participants")
        .await
        .map_err(|error| {
            error!("Redis error: {error:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let participants = participants
        .into_iter()
        .map(|(user_id, username)| PipelineParticipant {
            id: user_id,
            name: username,
        })
        .collect::<Vec<PipelineParticipant>>();

    let pipeline = Pipeline {
        id: pipeline[0].pipeline_id,
        name: pipeline[0].name.clone(),
        description: pipeline[0].description.clone(),
        nodes,
        connections,
        participants,
    };

    Ok(Json(pipeline))
}
