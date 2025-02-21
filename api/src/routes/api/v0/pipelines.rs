use super::{
    events::to_pipeline_participant_redis_key,
    pipeline_nodes::{Input, Output, PipelineConnection, PipelineNode},
};
use crate::routes::api::v0::pipeline_triggers::PipelineTrigger;
use crate::{
    app_state::{DatabaseConnection, JetStream, RedisConnection},
    extractors::session::Session,
    utils::internal_error,
};
use axum::{extract::Path, Json};
use axum_extra::extract::Query;
use db::dtos::{self, PipelineExecPayloadParams, PipelineTriggerConfig, PipelineTriggerConfigV0};
use hyper::StatusCode;
use redis::AsyncCommands;
use serde::Serialize;
use std::collections::HashSet;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Serialize)]
pub struct PipelineListItem {
    id: Uuid,
    name: String,
    description: Option<String>,
}

pub async fn list(
    DatabaseConnection(mut conn): DatabaseConnection,
) -> Result<Json<Vec<PipelineListItem>>, StatusCode> {
    let pipelines = sqlx::query!(
        r#"
        SELECT
            id, name, description
        FROM
            pipelines
        "#
    )
    .fetch_all(&mut *conn)
    .await
    .map(|rows| {
        rows.iter()
            .map(|row| PipelineListItem {
                id: row.id,
                name: row.name.clone(),
                description: row.description.clone(),
            })
            .collect::<Vec<PipelineListItem>>()
    })
    .map_err(|error| {
        error!("Database error: {error:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(pipelines))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCreateParams {
    team_id: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCreateResponse {
    id: Uuid,
}

pub async fn create(
    DatabaseConnection(mut conn): DatabaseConnection,
    Session(_): Session,
    Json(params): Json<PipelineCreateParams>,
) -> Result<Json<PipelineCreateResponse>, StatusCode> {
    let pipeline = sqlx::query!(
        r#"
        INSERT INTO
            pipelines (name, description, team_id)
        VALUES
            ($1, $2, $3)
        RETURNING
            id
        "#,
        params.name,
        params.description,
        params.team_id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    let trigger_config = serde_json::to_value(PipelineTriggerConfig::V0(PipelineTriggerConfigV0 {
        allow_manual_execution: true,
    }))
    .map_err(internal_error)?;

    let _ = sqlx::query!(
        r#"
        INSERT INTO
            pipeline_triggers (pipeline_id, coords, config)
        VALUES
            ($1, $2, $3)
        "#,
        pipeline.id,
        serde_json::json!({
            "x": -154,
            "y": -112,
        }),
        trigger_config
    )
    .execute(&mut *conn)
    .await
    .map_err(internal_error)?;

    Ok(Json(PipelineCreateResponse { id: pipeline.id }))
}

#[derive(Debug, Serialize)]
pub struct PipelineParticipant {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineDetails {
    id: Uuid,
    name: String,
    description: Option<String>,
    trigger: PipelineTrigger,
    nodes: Vec<PipelineNode>,
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
) -> Result<Json<PipelineDetails>, StatusCode> {
    let pipeline = sqlx::query!(
        r#"
        SELECT
            p.id, p.name, p.description,
            pt.id AS trigger_id, pt.coords as trigger_coords, pt.config AS trigger_config
        FROM
            pipelines p
        LEFT JOIN
            pipeline_triggers pt ON pt.pipeline_id = p.id
        WHERE
            p.id = $1
        "#,
        id
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(internal_error)?;

    let pipeline_nodes = sqlx::query!(
        r#"
        SELECT
            pn.id, pn.node_id, pn.node_version, pn.trigger_id, pn.coords,
            pni.id AS "input_id?", pni.key as "input_key?", pno.id AS "output_id?", pno.key AS "output_key?",
            pc.id AS "connection_id?", pc.to_pipeline_node_input_id AS "to_pipeline_node_input_id?", pc.from_pipeline_node_output_id AS "from_pipeline_node_output_id?"
        FROM
            pipeline_nodes pn
        LEFT JOIN
            pipeline_node_inputs pni ON pni.pipeline_node_id = pn.id
        LEFT JOIN
            pipeline_node_outputs pno ON pno.pipeline_node_id = pn.id
        LEFT JOIN
            pipeline_node_connections pc ON pc.to_pipeline_node_input_id = pni.id OR pc.from_pipeline_node_output_id = pno.id
        WHERE
            pn.pipeline_id = $1
        "#,
        pipeline.id
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(internal_error)?;

    let mut seen_nodes = HashSet::new();
    let mut nodes_map = std::collections::HashMap::new();

    for row in &pipeline_nodes {
        if seen_nodes.insert(row.node_id) {
            nodes_map.insert(
                row.node_id,
                PipelineNode {
                    id: row.id,
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

    for row in &pipeline_nodes {
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

    let nodes: Vec<PipelineNode> = nodes_map.into_values().collect();

    let mut seen_connections = HashSet::new();
    let connections = pipeline_nodes
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

    let pipeline = PipelineDetails {
        id: pipeline.id,
        name: pipeline.name.clone(),
        description: pipeline.description.clone(),
        trigger: PipelineTrigger {
            id: pipeline.trigger_id,
            coords: pipeline.trigger_coords.clone(),
            config: pipeline.trigger_config.clone(),
        },
        nodes,
        connections,
        participants,
    };

    Ok(Json(pipeline))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineTriggerResponse {
    pipeline_exec_id: Uuid,
}

pub async fn trigger(
    Path(id): Path<Uuid>,
    DatabaseConnection(mut db): DatabaseConnection,
    JetStream(jetstream): JetStream,
    Session(_): Session,
    Json(params): Json<PipelineExecPayloadParams>,
) -> Result<Json<PipelineTriggerResponse>, StatusCode> {
    info!("Received request to trigger pipeline with id: {id}");

    let pipeline_exec = sqlx::query!(
        r"
        INSERT INTO pipeline_execs (pipeline_id)
        VALUES ($1)
        RETURNING id
        ",
        id,
    )
    .fetch_one(&mut *db)
    .await
    .map_err(internal_error)?;

    info!(
        "Pipeline execution record created with id: {} and params: {params:?}",
        pipeline_exec.id
    );

    let payload = serde_json::to_string(&dtos::PipelineExecPayload {
        pipeline_id: id,
        pipeline_exec_id: pipeline_exec.id,
        params,
    })
    .map_err(internal_error)?;

    jetstream
        .publish("pipeline.exec", payload.into())
        .await
        .map_err(internal_error)?;

    Ok(Json(PipelineTriggerResponse {
        pipeline_exec_id: pipeline_exec.id,
    }))
}
