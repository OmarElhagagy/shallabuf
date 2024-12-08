use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SelectInput {
    pub value: String,
    pub label: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub enum NodeInputType {
    Text {
        default: Option<String>,
    },
    Select {
        options: Vec<SelectInput>,
        default: Option<String>,
    },
    Binary,
}

#[derive(Serialize, Deserialize)]
pub struct NodeInput {
    pub name: String,
    pub input: NodeInputType,
    pub label: Option<HashMap<String, String>>,
    pub required: bool,
    pub description: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub enum NodeOutputType {
    Text,
    Status,
}

#[derive(Serialize, Deserialize)]
pub struct NodeConfigV0 {
    pub inputs: Vec<NodeInput>,
    pub outputs: Vec<NodeOutputType>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum NodeConfig {
    V0(NodeConfigV0),
}
