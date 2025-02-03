use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PipelinePlanPayload {
    pub pipeline_exec_id: Uuid,
    pub pipeline_node_exec_id: Option<Uuid>,
}
