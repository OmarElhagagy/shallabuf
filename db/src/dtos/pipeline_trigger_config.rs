use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PipelineTriggerConfigV0 {
    pub allow_manual_execution: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum PipelineTriggerConfig {
    V0(PipelineTriggerConfigV0),
}
