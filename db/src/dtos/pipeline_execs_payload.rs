use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::NodeContainerType;

pub type PipelineExecPayloadParams = HashMap<Uuid, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNodeExecPayload {
    pub pipeline_execs_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub container_type: NodeContainerType,
    pub path: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ExecutionOutcome {
    Success(serde_json::Value),
    Failure(String),
}

#[derive(Serialize, Deserialize)]
pub struct PipelineNodeExecResultPayload {
    pub pipeline_exec_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub outcome: ExecutionOutcome,
}

#[derive(Serialize, Deserialize)]
pub struct PipelineExecPayload {
    pub pipeline_id: Uuid,
    pub pipeline_exec_id: Uuid,
    pub params: PipelineExecPayloadParams,
}
