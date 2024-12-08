use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type PipelineExecPayloadParams = HashMap<Uuid, serde_json::Value>;

#[derive(Serialize, Deserialize)]
pub struct PipelineExecPayload {
    pub pipeline_id: Uuid,
    pub pipeline_exec_id: Uuid,
    pub params: PipelineExecPayloadParams,
}
