use db::dtos;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

#[derive(Clone)]
pub struct PipelineRun {
    pub graph: DiGraph<Uuid, ()>,
    pub payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload>,
    pub nodes_exec_results: HashMap<Uuid, serde_json::Value>,
}

impl PipelineRun {
    pub fn new(
        graph: DiGraph<Uuid, ()>,
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

    pub fn next_nodes_to_execute(&self) -> Vec<dtos::PipelineNodeExecPayload> {
        self.graph
            .node_indices()
            .filter_map(|node_index| {
                let pipeline_node_id = self.graph[node_index];
                let pipeline_node_exec_id =
                    self.payloads.get(&pipeline_node_id)?.pipeline_node_exec_id;

                if self.nodes_exec_results.contains_key(&pipeline_node_exec_id) {
                    return None;
                }

                let is_root_node = self
                    .graph
                    .edges_directed(node_index, petgraph::Incoming)
                    .count()
                    == 0;

                let all_parents_have_result = self
                    .graph
                    .edges_directed(node_index, petgraph::Incoming)
                    .filter_map(|edge| {
                        let parent_index = edge.source();
                        let parent_node_id = self.graph[parent_index];

                        self.payloads
                            .get(&parent_node_id)
                            .map(|node_exec_payload| node_exec_payload.pipeline_node_exec_id)
                    })
                    .all(|parent_node_exec_id| {
                        self.nodes_exec_results.contains_key(&parent_node_exec_id)
                    });

                // Either should be a root node (a node without parents) and not executed yet
                // Or one of the children with parents which has results (means already executed)
                if is_root_node || all_parents_have_result {
                    self.payloads.get(&pipeline_node_id).cloned()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
