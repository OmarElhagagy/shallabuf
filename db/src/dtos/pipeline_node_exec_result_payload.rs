use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PipelineNodeExecResultPayload {
    pub pipeline_execs_id: Uuid,
    pub pipeline_node_exec_id: Uuid,
    pub result: serde_json::Value,
}
