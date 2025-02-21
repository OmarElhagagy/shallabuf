use db::dtos::{self, ExecStatus, PipelinePlanPayload};
use dotenvy::dotenv;
use futures::StreamExt;
use petgraph::graph::DiGraph;
use pipeline_run::{GraphEdgeConnection, PipelineRun};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::{collections::HashMap, env, process};
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};
use uuid::Uuid;

mod pipeline_run;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), async_nats::Error> {
    dotenv().ok();

    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    let (loki_layer, loki_task) = tracing_loki::builder()
        .label("host", "mine")
        .expect("Failed to create Loki layer")
        .extra_field("pid", format!("{}", process::id()))
        .expect("Failed to add extra field to Loki layer")
        .build_url(
            env::var("LOKI_URL")
                .expect("LOKI_URL must be set")
                .parse()
                .expect("Failed to parse Loki URL"),
        )
        .expect("Failed to build Loki layer");

    tokio::spawn(loki_task);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(loki_layer)
        .init();

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pg_pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let runs = Arc::new(RwLock::new(HashMap::<Uuid, PipelineRun>::new()));

    // --- Pipeline start execution ---
    let nats_client_clone = nats_client.clone();
    let runs_clone = Arc::clone(&runs);
    let mut pipeline_execs_subscriber = nats_client_clone.subscribe("pipeline.exec").await?;
    let pg_pool_clone = pg_pool.clone();

    tokio::spawn(async move {
        let pg_pool = pg_pool_clone;

        while let Some(message) = pipeline_execs_subscriber.next().await {
            let payload =
                match serde_json::from_slice::<dtos::PipelineExecPayload>(&message.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to deserialize message payload: {error:?}");
                        continue;
                    }
                };

            match sqlx::query!(
                r#"
                UPDATE
                    pipeline_execs
                SET
                    status = $1,
                    started_at = NOW()
                WHERE
                    id = $2
                "#,
                dtos::ExecStatus::Running as ExecStatus,
                payload.pipeline_exec_id
            )
            .execute(&pg_pool)
            .await
            {
                Ok(_) => {
                    info!(
                        "Pipeline execution status updated to 'running', pipeline_execs_id: {}",
                        payload.pipeline_exec_id
                    );
                }
                Err(error) => {
                    error!("Failed to update pipeline execution record: {error:?}, pipeline_exec_id: {}", payload.pipeline_exec_id);
                    continue;
                }
            }

            let pipeline_nodes = match sqlx::query!(
                r#"
                SELECT
                    pn.id,
                    pn.node_version,
                    n.publisher_name,
                    n.identifier_name,
                    n.config,
                    n.container_type::TEXT AS container_type
                FROM
                    pipeline_nodes pn
                JOIN
                    nodes n ON pn.node_id = n.id
                WHERE
                    pn.pipeline_id = $1
                "#,
                payload.pipeline_id
            )
            .fetch_all(&pg_pool)
            .await
            {
                Ok(nodes) => nodes,
                Err(error) => {
                    error!("Failed to fetch pipeline nodes: {error:?}");
                    continue;
                }
            };

            let mut graph = DiGraph::new();
            let mut node_index_map = std::collections::HashMap::new();
            for pn in &pipeline_nodes {
                let idx = graph.add_node(pn.id);
                node_index_map.insert(pn.id, idx);
            }
            let node_ids: Vec<Uuid> = pipeline_nodes.iter().map(|pn| pn.id).collect();

            let connections = match sqlx::query!(
                r#"
                SELECT
                    pno.pipeline_node_id AS "from_pipeline_node_id!: Uuid",
                    pno.key AS "from_key",
                    pni.pipeline_node_id AS "to_pipeline_node_id!: Uuid",
                    pni.key AS "to_key"
                FROM
                    pipeline_node_connections pnc
                JOIN
                    pipeline_node_outputs pno ON pno.id = pnc.from_pipeline_node_output_id
                JOIN
                    pipeline_node_inputs pni ON pni.id = pnc.to_pipeline_node_input_id
                WHERE
                    pno.pipeline_node_id = ANY($1) AND pni.pipeline_node_id = ANY($1)
                "#,
                &node_ids
            )
            .fetch_all(&pg_pool)
            .await
            {
                Ok(conns) => conns,
                Err(error) => {
                    error!("Failed to fetch pipeline node connections: {error:?}");
                    continue;
                }
            };

            for conn in connections {
                if let (Some(&from_idx), Some(&to_idx)) = (
                    node_index_map.get(&conn.from_pipeline_node_id),
                    node_index_map.get(&conn.to_pipeline_node_id),
                ) {
                    graph.add_edge(
                        from_idx,
                        to_idx,
                        GraphEdgeConnection {
                            source: conn.from_key,
                            target: conn.to_key,
                        },
                    );
                }
            }

            let mut pipeline_node_execs_payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload> =
                HashMap::new();

            match sqlx::query!(
                r#"
                INSERT INTO
                    pipeline_node_execs (pipeline_exec_id, pipeline_node_id)
                SELECT
                    pipeline_exec_id, pipeline_node_id
                FROM
                    UNNEST($1::uuid[], $2::uuid[]) AS a(pipeline_exec_id, pipeline_node_id)
                RETURNING
                    pipeline_node_execs.id, pipeline_node_execs.pipeline_node_id
                "#,
                &vec![payload.pipeline_exec_id; node_index_map.keys().len()] as &[Uuid],
                &node_index_map.keys().copied().collect::<Vec<Uuid>>()
            )
            .fetch_all(&pg_pool)
            .await
            {
                Ok(result) => {
                    for row in result {
                        let Some(pipeline_node) = pipeline_nodes
                            .iter()
                            .find(|pn| pn.id == row.pipeline_node_id)
                        else {
                            error!("Pipeline node not found for ID: {}", row.pipeline_node_id);
                            continue;
                        };

                        let Some(container_type) = pipeline_node.container_type.as_ref() else {
                            error!(
                                "Container type not found for pipeline node: {}",
                                row.pipeline_node_id
                            );
                            continue;
                        };

                        // Merge payload params with defaults from node config.
                        let user_params_value = payload
                            .params
                            .get(&row.pipeline_node_id)
                            .cloned()
                            .unwrap_or_default();

                        let user_params: std::collections::HashMap<String, String> =
                            if let serde_json::Value::Object(map) = user_params_value {
                                map.into_iter()
                                    .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
                                    .collect()
                            } else {
                                std::collections::HashMap::new()
                            };

                        let default_params: std::collections::HashMap<String, String> =
                            match serde_json::from_value(pipeline_node.config.clone()) {
                                Ok(config) => {
                                    let dtos::NodeConfig::V0(v0) = config;
                                    let mut defaults = std::collections::HashMap::new();

                                    for input in v0.inputs {
                                        match input.input {
                                            dtos::NodeInputType::Text { default }
                                            | dtos::NodeInputType::Select { default, .. } => {
                                                if let Some(val) = default {
                                                    defaults.insert(input.key.clone(), val);
                                                }
                                            }
                                            dtos::NodeInputType::Binary => {}
                                        }
                                    }

                                    defaults
                                }
                                Err(_) => std::collections::HashMap::new(),
                            };

                        let mut merged_params = default_params;
                        merged_params.extend(user_params);

                        // Convert merged parameters to a serde_json::Value.
                        let merged_params_json =
                            serde_json::to_value(merged_params).unwrap_or_default();

                        pipeline_node_execs_payloads.insert(
                            row.pipeline_node_id,
                            dtos::PipelineNodeExecPayload {
                                pipeline_execs_id: payload.pipeline_exec_id,
                                pipeline_node_exec_id: row.id,
                                container_type: container_type.into(),
                                path: format!(
                                    "{}@{}:{}",
                                    pipeline_node.publisher_name,
                                    pipeline_node.identifier_name,
                                    pipeline_node.node_version
                                ),
                                params: merged_params_json,
                            },
                        );
                    }
                }
                Err(error) => {
                    error!("Failed to insert pipeline node exec: {error:?}");
                    continue;
                }
            }

            let pipeline_run = PipelineRun::new(graph, pipeline_node_execs_payloads);
            let mut runs = runs_clone.write().await;
            runs.insert(payload.pipeline_exec_id, pipeline_run.clone());

            let nats_payload = match serde_json::to_string(&PipelinePlanPayload {
                pipeline_exec_id: payload.pipeline_exec_id,
                pipeline_node_exec_id: None,
            }) {
                Ok(payload) => payload,
                Err(error) => {
                    error!("Failed to serialize payload for PipelinePlanPayload: {error:?}");
                    continue;
                }
            };

            if let Err(error) = nats_client_clone
                .publish("pipeline.plan", nats_payload.into())
                .await
            {
                error!("Failed to publish message to JetStream: {error:?}");
            } else {
                info!(
                    "Published message to JetStream for pipeline_exec_id: {}",
                    payload.pipeline_exec_id
                );
            }
        }
    });

    // --- Pipeline planning ---
    let nats_client_clone = nats_client.clone();
    let runs_clone = Arc::clone(&runs);
    let mut pipeline_plan_subscriber = nats_client_clone.subscribe("pipeline.plan").await?;
    let pg_pool_clone = pg_pool.clone();

    tokio::spawn(async move {
        let pg_pool = pg_pool_clone;

        while let Some(message) = pipeline_plan_subscriber.next().await {
            let payload =
                match serde_json::from_slice::<dtos::PipelinePlanPayload>(&message.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to deserialize message payload: {error:?}");
                        continue;
                    }
                };

            let runs = runs_clone.read().await;

            let Some(pipeline_run) = runs.get(&payload.pipeline_exec_id) else {
                error!(
                    "Pipeline run not found for ID: {}",
                    payload.pipeline_exec_id
                );

                continue;
            };

            let next_nodes_to_be_executed =
                pipeline_run.next_nodes_to_execute(payload.pipeline_node_exec_id);

            if next_nodes_to_be_executed.is_empty() && pipeline_run.is_finished() {
                info!("Pipeline run finished for ID: {}", payload.pipeline_exec_id);

                match sqlx::query!(
                    r#"
                    UPDATE
                        pipeline_execs
                    SET
                        status = $1,
                        finished_at = NOW()
                    WHERE
                        id = $2;
                    "#,
                    dtos::ExecStatus::Completed as ExecStatus,
                    payload.pipeline_exec_id
                )
                .execute(&pg_pool)
                .await
                {
                    Ok(_) => {
                        info!("Pipeline execution status updated to 'completed', pipeline_execs_id: {}", payload.pipeline_exec_id);
                    }
                    Err(error) => {
                        error!(
                            "Failed to update pipeline execution status, error {error:?}, pipeline_exec_id: {}",
                            payload.pipeline_exec_id
                        );
                    }
                }

                continue;
            }

            for payload in next_nodes_to_be_executed {
                info!(
                    "Publishing message to JetStream for pipeline_node_exec_id: {} with payload: {payload:?}",
                    payload.pipeline_node_exec_id
                );

                match sqlx::query!(
                    r#"
                    UPDATE
                        pipeline_node_execs
                    SET
                        status = $1,
                        started_at = NOW()
                    WHERE
                        id = $2;
                    "#,
                    dtos::ExecStatus::Running as ExecStatus,
                    payload.pipeline_node_exec_id
                )
                .execute(&pg_pool)
                .await
                {
                    Ok(_) => {}
                    Err(error) => {
                        error!("Failed to update pipeline node exec status: {error:?}");
                        continue;
                    }
                }

                let nats_payload = match serde_json::to_string(&payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to serialize payload: {error:?}");
                        continue;
                    }
                };

                if let Err(error) = nats_client_clone
                    .publish("pipeline.node.exec", nats_payload.into())
                    .await
                {
                    error!("Failed to publish message to JetStream: {error:?}");
                } else {
                    info!(
                        "Published message to JetStream for pipeline_node_exec_id: {}",
                        payload.pipeline_node_exec_id
                    );
                }
            }
        }
    });

    // --- Pipeline node execution result ---
    let nats_client_clone = nats_client.clone();
    let runs_clone = Arc::clone(&runs);
    let mut pipeline_node_exec_result_subscriber =
        nats_client_clone.subscribe("pipeline.node.result").await?;
    let pg_pool_clone = pg_pool.clone();

    tokio::spawn(async move {
        let pg_pool = pg_pool_clone;
        let nats_client = nats_client_clone;

        while let Some(message) = pipeline_node_exec_result_subscriber.next().await {
            let payload = match serde_json::from_slice::<dtos::PipelineNodeExecResultPayload>(
                &message.payload,
            ) {
                Ok(payload) => payload,
                Err(error) => {
                    error!("Failed to deserialize message payload: {error:?}");
                    continue;
                }
            };

            let mut runs = runs_clone.write().await;

            let Some(pipeline_run) = runs.get_mut(&payload.pipeline_exec_id) else {
                error!(
                    "Pipeline run not found for ID: {}",
                    payload.pipeline_exec_id
                );

                continue;
            };

            let (exec_status, exec_result) = match payload.outcome {
                dtos::ExecutionOutcome::Failure(error) => {
                    error!("Pipeline node exec failed: {error:?}");
                    (ExecStatus::Failed, None)
                }
                dtos::ExecutionOutcome::Success(result) => (ExecStatus::Completed, Some(result)),
            };

            match sqlx::query!(
                r#"
                UPDATE
                    pipeline_node_execs
                SET
                    status = $1,
                    result = $2,
                    finished_at = NOW()
                WHERE
                    id = $3;
                "#,
                exec_status.clone() as ExecStatus,
                exec_result,
                payload.pipeline_node_exec_id
            )
            .execute(&pg_pool)
            .await
            {
                Ok(_) => {}
                Err(error) => {
                    error!("Failed to update pipeline node exec status: {error:?}");
                    continue;
                }
            }

            // There's nothing to be executed after a failed node,
            // All the downstream nodes executions will be cancelled
            if exec_status == ExecStatus::Failed {
                continue;
            }

            if let Some(exec_result) = exec_result {
                pipeline_run.update_node_exec_result(payload.pipeline_node_exec_id, exec_result);
            }

            let nats_payload = match serde_json::to_string(&PipelinePlanPayload {
                pipeline_exec_id: payload.pipeline_exec_id,
                pipeline_node_exec_id: payload.pipeline_node_exec_id.into(),
            }) {
                Ok(payload) => payload,
                Err(error) => {
                    error!("Failed to serialize payload for PipelinePlanPayload: {error:?}");
                    continue;
                }
            };

            if let Err(error) = nats_client
                .publish("pipeline.plan", nats_payload.into())
                .await
            {
                error!("Failed to publish message to JetStream: {error:?}");
            } else {
                info!(
                    "Published message to JetStream under pipeline.plan subject for pipeline_exec_id: {}",
                    payload.pipeline_exec_id
                );
            }
        }
    });

    ctrl_c().await?;

    Ok(())
}
