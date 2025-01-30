use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Debug, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "node_container_type", rename_all = "snake_case")]
pub enum NodeContainerType {
    Docker,
    Wasm,
}

impl From<&std::string::String> for NodeContainerType {
    fn from(s: &std::string::String) -> Self {
        match s.as_str() {
            "docker" => NodeContainerType::Docker,
            "wasm" => NodeContainerType::Wasm,
            _ => panic!("Invalid node container type"),
        }
    }
}
