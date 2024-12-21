use sea_orm::entity::prelude::*;
use sea_orm::FromQueryResult;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "node_container_type"
)]
pub enum NodeContainerType {
    #[sea_orm(string_value = "wasm")]
    Wasm,
    #[sea_orm(string_value = "docker")]
    Docker,
}

#[derive(FromQueryResult)]
pub struct PipelineNodesWithConnections {
    pub id: Uuid,
    pub node_version: String,
    pub publisher_name: String,
    pub name: String,
    pub container_type: Option<String>,
    pub from_pipeline_node_output_id: Option<Uuid>,
}
