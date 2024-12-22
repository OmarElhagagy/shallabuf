use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "exec_status", rename_all = "lowercase")]
pub enum ExecStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
