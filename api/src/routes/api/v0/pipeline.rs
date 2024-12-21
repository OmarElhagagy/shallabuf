use std::collections::HashSet;

use axum::{extract::Path, Json};
use axum_extra::extract::Query;
use hyper::StatusCode;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    app_state::{DatabaseConnection, RedisConnection},
    extractors::session::Session,
    utils::internal_error,
};

use super::events::to_pipeline_participant_redis_key;

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    id: Uuid,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    id: Uuid,
    key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    id: Uuid,
    node_id: Uuid,
    node_version: String,
    trigger_id: Option<Uuid>,
    coords: serde_json::Value,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

#[derive(Debug, Serialize)]
pub struct PipelineConnection {
    id: Uuid,
    to_pipeline_node_input_id: Uuid,
    from_pipeline_node_output_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct PipelineParticipant {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pipeline {
    id: Uuid,
    name: String,
    description: Option<String>,
    nodes: Vec<Node>,
    connections: Vec<PipelineConnection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    participants: Option<Vec<PipelineParticipant>>,
}

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
            pn.id AS pipeline_node_id, pn.node_id, pn.node_version, pn.trigger_id, pn.coords,
            pni.id AS "input_id?", pni.key as "input_key?", pno.id AS "output_id?", pno.key as "output_key?",
            pc.id AS "connection_id?", pc.to_pipeline_node_input_id as "to_pipeline_node_input_id?", pc.from_pipeline_node_output_id as "from_pipeline_node_output_id?"
        FROM
            pipelines p
        LEFT JOIN
            pipeline_nodes pn ON pn.pipeline_id = p.id
        LEFT JOIN
            pipeline_node_inputs pni ON pni.pipeline_node_id = pn.id
        LEFT JOIN
            pipeline_node_outputs pno ON pno.pipeline_node_id = pn.id
        LEFT JOIN
            pipeline_node_connections pc ON pc.to_pipeline_node_input_id = pni.id OR pc.from_pipeline_node_output_id = pno.id
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
    let mut nodes_map = std::collections::HashMap::new();

    for row in &pipeline {
        if seen_nodes.insert(row.node_id) {
            nodes_map.insert(
                row.node_id,
                Node {
                    id: row.pipeline_node_id,
                    node_id: row.node_id,
                    node_version: row.node_version.clone(),
                    trigger_id: row.trigger_id,
                    coords: row.coords.clone(),
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                },
            );
        }
    }

    for row in &pipeline {
        if let Some(node) = nodes_map.get_mut(&row.node_id) {
            if let Some(input_id) = row.input_id {
                if !node.inputs.iter().any(|input| input.id == input_id) {
                    node.inputs.push(Input {
                        id: input_id,
                        key: row.input_key.clone().unwrap(),
                    });
                }
            }

            if let Some(output_id) = row.output_id {
                if !node.outputs.iter().any(|output| output.id == output_id) {
                    node.outputs.push(Output {
                        id: output_id,
                        key: row.output_key.clone().unwrap(),
                    });
                }
            }
        }
    }

    let nodes: Vec<Node> = nodes_map.into_values().collect();

    let mut seen_connections = HashSet::new();
    let connections = pipeline
        .iter()
        .filter_map(|row| {
            if let (
                Some(connection_id),
                Some(to_pipeline_node_input_id),
                Some(from_pipeline_node_output_id),
            ) = (
                row.connection_id,
                row.to_pipeline_node_input_id,
                row.from_pipeline_node_output_id,
            ) {
                if seen_connections.insert(row.connection_id) {
                    Some(PipelineConnection {
                        id: connection_id,
                        to_pipeline_node_input_id,
                        from_pipeline_node_output_id,
                    })
                } else {
                    None
                }
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
