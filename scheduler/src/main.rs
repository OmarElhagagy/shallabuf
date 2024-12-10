use db::{dtos, ConnectionTrait};
use db::{
    entities::pipeline_nodes,
    sea_orm::{ConnectOptions, Database},
};
use futures::StreamExt;
use models::{NodeContainerType, PipelineNodesWithConnections};
use petgraph::graph::DiGraph;
use pipeline_run::PipelineRun;
use sea_orm::EntityTrait;
use sea_orm::Statement;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};
use uuid::Uuid;

mod models;
mod pipeline_run;

impl From<NodeContainerType> for db::dtos::NodeContainerType {
    fn from(container_type: NodeContainerType) -> db::dtos::NodeContainerType {
        match container_type {
            NodeContainerType::Wasm => db::dtos::NodeContainerType::Wasm,
            NodeContainerType::Docker => db::dtos::NodeContainerType::Docker,
        }
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), async_nats::Error> {
    let filter_layer = EnvFilter::from_default_env();
    let fmt_layer = fmt::layer().with_target(false).with_line_number(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connections = std::env::var("MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse::<u32>()
        .expect("MAX_CONNECTIONS must be a number");

    let mut options = ConnectOptions::new(database_url);
    options.max_connections(max_connections);
    let db = Database::connect(options).await?;

    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set");
    let nats_client = async_nats::connect(nats_url)
        .await
        .expect("Failed to connect to NATS");

    let runs = Arc::new(RwLock::new(HashMap::<Uuid, PipelineRun>::new()));

    let nats_client_clone = nats_client.clone();
    let runs_clone = Arc::clone(&runs);
    let mut pipeline_node_exec_result_subscriber =
        nats_client_clone.subscribe("pipeline.node.result").await?;

    tokio::spawn(async move {
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

            pipeline_run.update_node_exec_result(payload.pipeline_node_exec_id, payload.result);
            let nodes_to_be_executed = pipeline_run.next_nodes_to_execute();

            if nodes_to_be_executed.is_empty() {
                info!("Pipeline run finished for ID: {}", payload.pipeline_exec_id);
                continue;
            }

            for payload in nodes_to_be_executed {
                info!(
                    "Publishing message to JetStream for pipeline_node_exec_id: {}",
                    payload.pipeline_node_exec_id
                );

                let payload_bytes = match serde_json::to_string(&payload) {
                    Ok(payload) => payload.into(),
                    Err(error) => {
                        error!("Failed to serialize payload: {error:?}");
                        continue;
                    }
                };

                if let Err(error) = nats_client_clone
                    .publish("pipeline.node.exec", payload_bytes)
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

    let nats_client_clone = nats_client.clone();
    let runs_clone = Arc::clone(&runs);
    let mut pipeline_exec_subscriber = nats_client_clone.subscribe("pipeline.exec").await?;

    tokio::spawn(async move {
        while let Some(message) = pipeline_exec_subscriber.next().await {
            let payload =
                match serde_json::from_slice::<dtos::PipelineExecPayload>(&message.payload) {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to deserialize message payload: {error:?}");
                        continue;
                    }
                };

            let pipeline_nodes = match pipeline_nodes::Entity::find()
                .from_raw_sql(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    r"
                        SELECT
                            pipeline_nodes.id,
                            pipeline_nodes.node_version,
                            nodes.publisher_name,
                            nodes.name,
                            nodes.container_type::TEXT AS container_type,
                            pipeline_nodes_connections.to_node_id
                        FROM
                            pipeline_nodes
                        LEFT JOIN
                            nodes ON pipeline_nodes.node_id = nodes.id
                        LEFT JOIN
                            pipeline_nodes_connections
                            ON pipeline_nodes.id = pipeline_nodes_connections.from_node_id
                            OR pipeline_nodes.id = pipeline_nodes_connections.to_node_id
                        WHERE
                            pipeline_nodes.pipeline_id = $1;
                    ",
                    [payload.pipeline_id.into()],
                ))
                .into_model::<PipelineNodesWithConnections>()
                .all(&db)
                .await
            {
                Ok(nodes) => nodes,
                Err(error) => {
                    error!("Failed to fetch pipeline nodes: {error:?}");
                    continue;
                }
            };

            let mut graph = DiGraph::new();
            let mut graph_nodes = std::collections::HashMap::new();

            for pipeline_node in &pipeline_nodes {
                let (node_index, _) = *graph_nodes
                    .entry(pipeline_node.id)
                    .or_insert_with(|| (graph.add_node(pipeline_node.id), pipeline_node));

                if let Some(to_node_id) = pipeline_node.to_node_id {
                    let (child_index, _) = *graph_nodes
                        .entry(to_node_id)
                        .or_insert_with(|| (graph.add_node(to_node_id), pipeline_node));

                    if node_index != child_index {
                        graph.add_edge(node_index, child_index, ());
                    }
                }
            }

            let mut pipeline_nodes_exec_payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload> =
                HashMap::new();

            // Because sea-orm doesn't support RETURNING the inserted ids (plural) from INSERT INTO stmt, we need to use a raw SQL
            match db
                .query_all(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        r"
                            INSERT INTO
                                pipeline_nodes_exec (pipeline_exec_id, pipeline_node_id)
                            VALUES
                                {}
                            RETURNING
                                pipeline_nodes_exec.id, pipeline_nodes_exec.pipeline_node_id;
                        ",
                        (1..=graph_nodes.keys().len())
                            .map(|i| format!("(${}, ${})", i + i - 1, i + i))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    graph_nodes
                        .keys()
                        .flat_map(|pipeline_node_id| {
                            [payload.pipeline_exec_id.into(), (*pipeline_node_id).into()]
                        })
                        .collect::<Vec<_>>(),
                ))
                .await
            {
                Ok(result) => {
                    for row in result {
                        let pipeline_node_exec_id = match row.try_get_by_index(0) {
                            Ok(pipeline_node_exec_id) => pipeline_node_exec_id,
                            Err(error) => {
                                error!("Failed to get pipeline_nodes_exec_id: {error:?}");
                                continue;
                            }
                        };

                        let pipeline_node_id = match row.try_get_by_index(1) {
                            Ok(id) => id,
                            Err(error) => {
                                error!("Failed to get pipeline_node_id: {error:?}");
                                continue;
                            }
                        };

                        let Some((_, pipeline_node)) = graph_nodes.get(&pipeline_node_id) else {
                            error!("Pipeline node not found for ID: {pipeline_node_id}");
                            continue;
                        };

                        let Some(ref container_type_str) = pipeline_node.container_type else {
                            error!(
                                "Container type not found for pipeline node: {pipeline_node_id}"
                            );
                            continue;
                        };

                        let container_type = match container_type_str.as_str() {
                            "wasm" => dtos::NodeContainerType::Wasm,
                            "docker" => dtos::NodeContainerType::Docker,
                            _ => {
                                error!("Invalid container type: {}", container_type_str);
                                continue;
                            }
                        };

                        pipeline_nodes_exec_payloads.insert(
                            pipeline_node_id,
                            dtos::PipelineNodeExecPayload {
                                pipeline_exec_id: payload.pipeline_exec_id,
                                pipeline_node_exec_id,
                                container_type,
                                path: format!(
                                    "{}@{}:{}",
                                    pipeline_node.publisher_name,
                                    pipeline_node.name,
                                    pipeline_node.node_version
                                ),
                                params: payload
                                    .params
                                    .get(&pipeline_node_id)
                                    .cloned()
                                    .unwrap_or_default(),
                            },
                        );
                    }
                }
                Err(error) => {
                    error!("Failed to insert pipeline node exec: {error:?}");
                    continue;
                }
            };

            let pipeline_run = PipelineRun::new(graph, pipeline_nodes_exec_payloads);
            let mut runs = runs_clone.write().await;
            runs.insert(payload.pipeline_exec_id, pipeline_run.clone());

            let nodes_to_be_executed = pipeline_run.next_nodes_to_execute();

            if nodes_to_be_executed.is_empty() {
                // TODO: Publish pipeline run finished event (pipeline_exec_id) to JetStream and update pipeline_exec status
                // TODO: (maybe we need to indicate that pipeline isn't configured properly)
                debug!("Pipeline run is already finished");
                continue;
            }

            for payload in nodes_to_be_executed {
                info!(
                    "Publishing message to JetStream for pipeline_node_id: {}",
                    payload.pipeline_node_exec_id
                );

                let payload_bytes = match serde_json::to_string(&payload) {
                    Ok(payload) => payload.into(),
                    Err(error) => {
                        error!("Failed to serialize payload: {error:?}");
                        continue;
                    }
                };

                if let Err(error) = nats_client_clone
                    .publish("pipeline.node.exec", payload_bytes)
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

    ctrl_c().await?;

    Ok(())
}
