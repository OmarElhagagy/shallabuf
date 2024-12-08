use db::{dtos, ConnectionTrait};
use db::{entities::nodes, sea_orm::ColumnTrait};
use db::{
    entities::{pipeline_nodes, pipeline_nodes_connections},
    sea_orm::{ConnectOptions, Database},
};
use futures::StreamExt;
use models::PipelineNodesWithConnections;
use petgraph::graph::DiGraph;
use pipeline_run::PipelineRun;
use sea_orm::QuerySelect;
use sea_orm::{EntityTrait, JoinType, QueryFilter};
use sea_orm::{RelationTrait, Statement};
use std::collections::HashMap;
use tokio::signal::ctrl_c;
use tracing::{debug, error};
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};
use uuid::Uuid;

mod models;
mod pipeline_run;

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

    let mut runs = HashMap::new();
    let mut pipeline_exec_subscriber = nats_client.subscribe("pipeline.exec").await?;

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
                .select_only()
                .column(pipeline_nodes::Column::Id)
                .column(pipeline_nodes::Column::NodeVersion)
                .column(nodes::Column::PublisherName)
                .column(nodes::Column::Name)
                .column(nodes::Column::ContainerType)
                .column(pipeline_nodes_connections::Column::ToNodeId)
                .join(JoinType::LeftJoin, nodes::Relation::PipelineNodes.def())
                .join(
                    JoinType::LeftJoin,
                    pipeline_nodes_connections::Relation::PipelineNodes1.def(),
                )
                .join(
                    JoinType::LeftJoin,
                    pipeline_nodes_connections::Relation::PipelineNodes2.def(),
                )
                .filter(pipeline_nodes::Column::PipelineId.eq(payload.pipeline_id))
                .into_model::<PipelineNodesWithConnections>()
                .all(&db)
                .await
            {
                Ok(nodes) => nodes,
                Err(error) => {
                    error!("Failed to fetch pipeline tasks: {error:?}");
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

            // Because sea-orm doesn't support RETURNING the inserted id, we need to use raw SQL
            match db
                .query_all(Statement::from_sql_and_values(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        "INSERT INTO pipeline_node_exec VALUES ({}) RETURNING pipeline_node_exec.id, pipeline_node_exec.pipeline_node_id;",
                        (1..=graph_nodes.keys().len()).map(|i| format!("${i}")).collect::<Vec<_>>().join(", "),
                    ),
                    graph_nodes.keys().map(|&uuid| uuid.into()),
                ))
                .await
            {
                Ok(result) => {
                    for row in result {
                        let pipeline_node_exec_id = match row.try_get_by_index(0) {
                            Ok(pipeline_node_exec_id) => pipeline_node_exec_id,
                            Err(error) => {
                                error!("Failed to get pipeline_node_exec_id: {error:?}");
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

                        pipeline_nodes_exec_payloads.insert(
                            pipeline_node_id,
                            dtos::PipelineNodeExecPayload {
                                pipeline_exec_id: payload.pipeline_exec_id,
                                pipeline_node_exec_id,
                                container_type: pipeline_node.container_type.clone().into(),
                                path: format!("{}@{}:{}", pipeline_node.publisher_name, pipeline_node.name, pipeline_node.node_version),
                                params: payload.params.get(&pipeline_node_id).cloned().unwrap_or_default(),
                            }
                        );
                    }
                }
                Err(error) => {
                    error!("Failed to insert pipeline node exec: {error:?}");
                    continue;
                }
            };

            let pipeline_run = PipelineRun::new(graph, pipeline_nodes_exec_payloads);
            runs.insert(payload.pipeline_exec_id, pipeline_run.clone());

            if pipeline_run.is_finished() {
                // TODO: Publish pipeline run finished event (pipeline_exec_id) to JetStream and update pipeline_exec status
                // TODO: (maybe we need to indicate that pipeline isn't configured properly)
                debug!("Pipeline run is already finished");
                continue;
            }

            for payload in pipeline_run.next_nodes_to_execute() {
                let payload = match serde_json::to_string(&payload) {
                    Ok(payload) => payload.into_bytes(),
                    Err(error) => {
                        error!("Failed to serialize payload: {error:?}");
                        continue;
                    }
                };

                if let Err(error) = nats_client
                    .publish("pipeline.node.exec", payload.into())
                    .await
                {
                    error!("Failed to publish message to JetStream: {error:?}");
                } else {
                    debug!("Published message to JetStream for pipeline_node_id",);
                }
            }
        }
    });

    ctrl_c().await?;

    Ok(())
}
