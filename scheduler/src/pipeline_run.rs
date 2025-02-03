use db::dtos;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct GraphEdgeConnection {
    pub source: String,
    pub target: String,
}

#[derive(Clone)]
pub struct PipelineRun {
    pub graph: DiGraph<Uuid, GraphEdgeConnection>,
    pub payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload>,
    pub nodes_exec_results: HashMap<Uuid, serde_json::Value>,
}

impl PipelineRun {
    pub fn new(
        graph: DiGraph<Uuid, GraphEdgeConnection>,
        payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload>,
    ) -> Self {
        Self {
            graph,
            payloads,
            nodes_exec_results: HashMap::new(),
        }
    }

    pub fn update_node_exec_result(&mut self, node_exec_id: Uuid, result: serde_json::Value) {
        self.nodes_exec_results.insert(node_exec_id, result);
    }

    pub fn next_nodes_to_execute(
        &self,
        parent_pipeline_node_exec_id: Option<Uuid>,
    ) -> Vec<dtos::PipelineNodeExecPayload> {
        self.graph
            .node_indices()
            .filter_map(|node_index| {
                let pipeline_node_id = self.graph[node_index];
                let pipeline_node_exec_id =
                    self.payloads.get(&pipeline_node_id)?.pipeline_node_exec_id;

                if self.nodes_exec_results.contains_key(&pipeline_node_exec_id) {
                    return None;
                }

                // If parent_pipeline_node_exec_id is provided, check if the current node is a child of the parent
                // If not, return None
                if let Some(parent_pipeline_node_exec_id) = parent_pipeline_node_exec_id {
                    let is_target_child = self
                        .graph
                        .edges_directed(node_index, petgraph::Incoming)
                        .any(|edge| {
                            let parent_index = edge.source();
                            let parent_node_id = self.graph[parent_index];

                            let parent_node_exec_id = self
                                .payloads
                                .get(&parent_node_id)
                                .map(|node_exec_payload| node_exec_payload.pipeline_node_exec_id)
                                .unwrap_or_default();

                            parent_node_exec_id == parent_pipeline_node_exec_id
                        });

                    if !is_target_child {
                        return None;
                    }
                }

                let is_root_node = self
                    .graph
                    .edges_directed(node_index, petgraph::Incoming)
                    .count()
                    == 0;

                let all_parents_results = self
                    .graph
                    .edges_directed(node_index, petgraph::Incoming)
                    .filter_map(|edge| {
                        let parent_index = edge.source();
                        let parent_node_id = self.graph[parent_index];

                        let pipeline_node_exec_id = self
                            .payloads
                            .get(&parent_node_id)
                            .map(|node_exec_payload| node_exec_payload.pipeline_node_exec_id)?;

                        let graph_edge_connection = edge.weight();

                        (pipeline_node_exec_id, graph_edge_connection).into()
                    });

                let all_parents_have_result =
                    all_parents_results.clone().all(|(parent_node_exec_id, _)| {
                        self.nodes_exec_results.contains_key(&parent_node_exec_id)
                    });

                // Either should be a root node (a node without parents) and not executed yet
                // Or one of the children with parents which has results (means already executed)
                if is_root_node || all_parents_have_result {
                    let mut payload = self.payloads.get(&pipeline_node_id).cloned();
                    let mut params = HashMap::new();

                    for (parent_node_exec_id, edge_connection) in all_parents_results {
                        if let Some(value) = self.nodes_exec_results.get(&parent_node_exec_id) {
                            if let Some(value) = value.get(edge_connection.source.as_str()) {
                                params.insert(edge_connection.target.clone(), value.clone());
                            }
                        }
                    }

                    // Merge computed params into payload.params (of type serde_json::Value)
                    if let Some(ref mut payload) = payload {
                        if let Some(existing_params) = payload.params.as_object_mut() {
                            for (k, v) in params {
                                existing_params.insert(k, v);
                            }
                        } else {
                            let map: serde_json::Map<String, serde_json::Value> =
                                params.into_iter().collect();

                            payload.params = serde_json::Value::Object(map);
                        }
                    }

                    payload
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn is_finished(&self) -> bool {
        self.next_nodes_to_execute(None).is_empty()
    }
}
