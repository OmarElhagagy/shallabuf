use sqlx::types::Uuid;

pub enum PipelineStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(sqlx::FromRow)]
pub struct PipelineExec {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: PipelineStatus,
    pub started_at: sqlx::types::time::Date,
    pub finished_at: sqlx::types::time::Date,
}
