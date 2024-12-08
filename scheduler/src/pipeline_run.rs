use db::dtos;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone)]
pub struct PipelineRun {
    pub graph: DiGraph<Uuid, ()>,
    pub payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload>,
    pub nodes_to_be_executed: HashSet<Uuid>,
    pub nodes_execution_results: HashMap<Uuid, serde_json::Value>,
}

impl PipelineRun {
    pub fn new(
        graph: DiGraph<Uuid, ()>,
        payloads: HashMap<Uuid, dtos::PipelineNodeExecPayload>,
    ) -> Self {
        let nodes_to_be_executed = graph
            .node_indices()
            .map(|node_index| graph[node_index])
            .collect();

        Self {
            graph,
            payloads,
            nodes_to_be_executed,
            nodes_execution_results: HashMap::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.nodes_to_be_executed.is_empty()
    }

    pub fn next_nodes_to_execute(&self) -> Vec<dtos::PipelineNodeExecPayload> {
        self.graph
            .node_indices()
            .filter_map(|node_index| {
                let pipeline_node_id = self.graph[node_index];

                if self.nodes_execution_results.contains_key(&pipeline_node_id) {
                    return None;
                }

                let parents: Vec<_> = self
                    .graph
                    .edges_directed(node_index, petgraph::Incoming)
                    .map(|edge| edge.source())
                    .collect();

                if parents.is_empty()
                    || parents.iter().all(|&parent_index| {
                        let parent_id = self.graph[parent_index];
                        self.nodes_execution_results.contains_key(&parent_id)
                    })
                {
                    self.payloads.get(&pipeline_node_id).cloned()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}
