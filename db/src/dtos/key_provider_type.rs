use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "key_provider_type", rename_all = "snake_case")]
pub enum KeyProviderType {
    Password,
    Github,
    Google,
    Facebook,
}
