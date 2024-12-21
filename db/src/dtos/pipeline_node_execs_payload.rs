use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::NodeContainerType;

// #[derive(Clone, Serialize, Deserialize)]
// pub enum NodeContainerType {
//     #[serde(rename = "wasm")]
//     Wasm,
//     #[serde(rename = "docker")]
//     Docker,
// }

// impl From<String> for NodeContainerType {
//     fn from(container_type: String) -> Self {
//         match container_type.as_str() {
//             "wasm" => NodeContainerType::Wasm,
//             "docker" => NodeContainerType::Docker,
//             _ => panic!("Invalid container type: {container_type}"),
//         }
//     }
// }

#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineNodeExecPayload {
    pub pipeline_execs_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub container_type: NodeContainerType,
    pub path: String,
    pub params: serde_json::Value,
}
