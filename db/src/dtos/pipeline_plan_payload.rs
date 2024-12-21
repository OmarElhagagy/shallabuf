use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct PipelinePlanPayload {
    pub pipeline_execs_id: Uuid,
}
