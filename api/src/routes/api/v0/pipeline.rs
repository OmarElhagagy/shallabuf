use std::collections::HashSet;

use axum::{extract::Path, Json};
use axum_extra::extract::Query;
use hyper::StatusCode;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    app_state::{DatabaseConnection, RedisConnection},
    extractors::session::Session,
    utils::internal_error,
    Pipeline, PipelineConnection, PipelineNode, PipelineParticipant,
};

use super::events::to_pipeline_participant_redis_key;

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticipantOption {
    IncludeMyself,
}

impl std::str::FromStr for ParticipantOption {
    type Err = ();

    fn from_str(input: &str) -> Result<ParticipantOption, Self::Err> {
        match input {
            "includeMyself" => Ok(ParticipantOption::IncludeMyself),
            _ => Err(()),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDetailsQuery {
    #[serde(default)]
    pub with_participants: Option<Vec<ParticipantOption>>,
}

#[allow(clippy::too_many_lines)]
pub async fn details(
    DatabaseConnection(mut conn): DatabaseConnection,
    RedisConnection(mut redis): RedisConnection,
    Session(session): Session,
    Path(id): Path<Uuid>,
    Query(query): Query<PipelineDetailsQuery>,
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
    .map_err(internal_error)?;

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

    let mut participants = None::<Vec<PipelineParticipant>>;

    if query.with_participants.is_some() {
        let raw_participants: Vec<(String, String)> = redis
            .hgetall(to_pipeline_participant_redis_key(id))
            .await
            .map_err(internal_error)?;

        participants = raw_participants
            .into_iter()
            .filter_map(|(user_id, username)| {
                let user_id = Uuid::parse_str(&user_id).ok()?;
                Some(PipelineParticipant {
                    id: user_id,
                    name: username,
                })
            })
            .collect::<Vec<PipelineParticipant>>()
            .into();

        if let Some(params) = query.with_participants.as_ref() {
            for param in params {
                if *param == ParticipantOption::IncludeMyself {
                    let current_user = PipelineParticipant {
                        id: session.user_id,
                        name: session.username.clone(),
                    };

                    if let Some(participants) = participants.as_mut() {
                        participants.push(current_user);
                    }
                }
            }
        }
    }

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
