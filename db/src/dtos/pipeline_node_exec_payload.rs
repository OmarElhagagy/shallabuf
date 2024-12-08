use crate::entities::sea_orm_active_enums::NodeContainerType as NodeContainerTypeActiveEnum;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub enum NodeContainerType {
    Wasm,
    Docker,
}

impl From<NodeContainerTypeActiveEnum> for NodeContainerType {
    fn from(container_type: NodeContainerTypeActiveEnum) -> Self {
        match container_type {
            NodeContainerTypeActiveEnum::Wasm => NodeContainerType::Wasm,
            NodeContainerTypeActiveEnum::Docker => NodeContainerType::Docker,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PipelineNodeExecPayload {
    pub pipeline_exec_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub container_type: NodeContainerType,
    pub path: String,
    pub params: serde_json::Value,
}
