use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, PartialEq, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "exec_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExecStatus {
    Pending,
    Running,
    Completed,
    Failed,
}
