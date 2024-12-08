//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use super::sea_orm_active_enums::ExecStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pipeline_exec")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub status: ExecStatus,
    pub created_at: DateTime,
    pub started_at: Option<DateTime>,
    pub finished_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::pipeline_nodes_exec::Entity")]
    PipelineNodesExec,
    #[sea_orm(
        belongs_to = "super::pipelines::Entity",
        from = "Column::PipelineId",
        to = "super::pipelines::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Pipelines,
}

impl Related<super::pipeline_nodes_exec::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PipelineNodesExec.def()
    }
}

impl Related<super::pipelines::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pipelines.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
