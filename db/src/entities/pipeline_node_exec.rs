//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use super::sea_orm_active_enums::ExecStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pipeline_node_exec")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pipeline_exec_id: Uuid,
    pub pipeline_node_id: Uuid,
    pub status: ExecStatus,
    pub result: Option<Json>,
    pub created_at: DateTime,
    pub started_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::pipeline_exec::Entity",
        from = "Column::PipelineExecId",
        to = "super::pipeline_exec::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PipelineExec,
    #[sea_orm(
        belongs_to = "super::pipeline_nodes::Entity",
        from = "Column::PipelineNodeId",
        to = "super::pipeline_nodes::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    PipelineNodes,
}

impl Related<super::pipeline_exec::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PipelineExec.def()
    }
}

impl Related<super::pipeline_nodes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PipelineNodes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
