use db::entities::sea_orm_active_enums::NodeContainerType;
use sea_orm::FromQueryResult;
use uuid::Uuid;

#[derive(FromQueryResult)]
pub struct PipelineNodesWithConnections {
    pub id: Uuid,
    pub node_version: String,
    pub publisher_name: String,
    pub name: String,
    pub container_type: NodeContainerType,
    pub to_node_id: Option<Uuid>,
}
