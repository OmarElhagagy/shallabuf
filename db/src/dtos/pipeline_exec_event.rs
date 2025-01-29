use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ExecStatus;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineExec {
    pub id: Uuid,
    #[serde(alias = "pipeline_id")]
    pub pipeline_id: Uuid,
    pub status: ExecStatus,
    #[serde(alias = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(alias = "started_at")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(alias = "finished_at")]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineNodeExec {
    pub id: Uuid,
    #[serde(alias = "pipeline_exec_id")]
    pub pipeline_exec_id: Uuid,
    #[serde(alias = "pipeline_node_id")]
    pub pipeline_node_id: Uuid,
    pub status: ExecStatus,
    pub result: serde_json::Value,
    #[serde(alias = "created_at")]
    pub created_at: DateTime<Utc>,
    #[serde(alias = "started_at")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(alias = "finished_at")]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum PipelineExecEvent {
    Pipeline(PipelineExec),
    Node(PipelineNodeExec),
}
