use extension::postgres::Type;
use sea_orm_migration::prelude::*;

use super::m20220101_000001_auth_schema::Teams;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
#[allow(clippy::too_many_lines)]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create 'visibility' enum type
        manager
            .create_type(
                Type::create()
                    .as_enum(Visibility::Enum)
                    .values([Visibility::Public, Visibility::Private])
                    .to_owned(),
            )
            .await?;

        // Create 'exec_status' enum type
        manager
            .create_type(
                Type::create()
                    .as_enum(ExecStatus::Enum)
                    .values([
                        ExecStatus::Pending,
                        ExecStatus::Running,
                        ExecStatus::Completed,
                        ExecStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create 'node_container_type' enum type
        manager
            .create_type(
                Type::create()
                    .as_enum(NodeContainerType::Enum)
                    .values([NodeContainerType::Wasm, NodeContainerType::Docker])
                    .to_owned(),
            )
            .await?;

        // Create 'templates' table
        manager
            .create_table(
                Table::create()
                    .table(Templates::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Templates::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Templates::Name).string().not_null())
                    .col(ColumnDef::new(Templates::Description).string())
                    .col(ColumnDef::new(Templates::Config).json().not_null())
                    .col(
                        ColumnDef::new(Templates::Visibility)
                            .custom(Visibility::Enum)
                            .default(Visibility::Public.to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Templates::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Templates::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipelines' table
        manager
            .create_table(
                Table::create()
                    .table(Pipelines::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Pipelines::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Pipelines::Name).string().not_null())
                    .col(ColumnDef::new(Pipelines::Description).string())
                    .col(ColumnDef::new(Pipelines::FromTemplateId).uuid())
                    .col(ColumnDef::new(Pipelines::TeamId).uuid().not_null())
                    .col(
                        ColumnDef::new(Pipelines::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Pipelines::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipelines-from_template_id")
                            .from(Pipelines::Table, Pipelines::FromTemplateId)
                            .to(Templates::Table, Templates::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipelines-team_id")
                            .from(Pipelines::Table, Pipelines::TeamId)
                            .to(Teams::Table, Teams::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipeline_triggers' table
        manager
            .create_table(
                Table::create()
                    .table(PipelineTriggers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineTriggers::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(PipelineTriggers::PipelineId)
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PipelineTriggers::Config).json().not_null())
                    .col(
                        ColumnDef::new(PipelineTriggers::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PipelineTriggers::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_triggers-pipeline_id")
                            .from(PipelineTriggers::Table, PipelineTriggers::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'nodes' table
        manager
            .create_table(
                Table::create()
                    .table(Nodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Nodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(Nodes::Name).string().not_null())
                    .col(ColumnDef::new(Nodes::PublisherName).string().not_null())
                    .col(ColumnDef::new(Nodes::Description).string())
                    .col(ColumnDef::new(Nodes::Config).json().not_null())
                    .col(
                        ColumnDef::new(Nodes::ContainerType)
                            .custom(NodeContainerType::Enum)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Nodes::Tags)
                            .array(ColumnType::Text)
                            .default("{}".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Nodes::Versions)
                            .array(ColumnType::Text)
                            .default("{\"latest\"}".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Nodes::Visibility)
                            .custom(Visibility::Enum)
                            .default("public".to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Nodes::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Nodes::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipeline_exec' table
        manager
            .create_table(
                Table::create()
                    .table(PipelineExec::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineExec::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(PipelineExec::PipelineId).uuid().not_null())
                    .col(
                        ColumnDef::new(PipelineExec::Status)
                            .custom(ExecStatus::Enum)
                            .default(ExecStatus::Pending.to_string())
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PipelineExec::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(PipelineExec::StartedAt).timestamp())
                    .col(ColumnDef::new(PipelineExec::FinishedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_exec-pipeline_id")
                            .from(PipelineExec::Table, PipelineExec::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipeline_nodes' table
        manager
            .create_table(
                Table::create()
                    .table(PipelineNodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineNodes::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(PipelineNodes::PipelineId).uuid().not_null())
                    .col(ColumnDef::new(PipelineNodes::NodeId).uuid().not_null())
                    .col(
                        ColumnDef::new(PipelineNodes::NodeVersion)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PipelineNodes::TriggerId).uuid())
                    .col(
                        ColumnDef::new(PipelineNodes::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PipelineNodes::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes-pipeline_id")
                            .from(PipelineNodes::Table, PipelineNodes::PipelineId)
                            .to(Pipelines::Table, Pipelines::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes-node_id")
                            .from(PipelineNodes::Table, PipelineNodes::NodeId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes-trigger_id")
                            .from(PipelineNodes::Table, PipelineNodes::TriggerId)
                            .to(PipelineTriggers::Table, PipelineTriggers::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipeline_nodes_exec' table
        manager
            .create_table(
                Table::create()
                    .table(PipelineNodesExec::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineNodesExec::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesExec::PipelineExecId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesExec::PipelineNodeId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesExec::Status)
                            .custom(ExecStatus::Enum)
                            .default(ExecStatus::Pending.to_string())
                            .not_null(),
                    )
                    .col(ColumnDef::new(PipelineNodesExec::Result).json())
                    .col(
                        ColumnDef::new(PipelineNodesExec::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(PipelineNodesExec::StartedAt).timestamp())
                    .col(ColumnDef::new(PipelineNodesExec::FinishedAt).timestamp())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes_exec-pipeline_exec_id")
                            .from(PipelineNodesExec::Table, PipelineNodesExec::PipelineExecId)
                            .to(PipelineExec::Table, PipelineExec::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes_exec-pipeline_node_id")
                            .from(PipelineNodesExec::Table, PipelineNodesExec::PipelineNodeId)
                            .to(PipelineNodes::Table, PipelineNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create 'pipeline_nodes_connections' table
        manager
            .create_table(
                Table::create()
                    .table(PipelineNodesConnections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PipelineNodesConnections::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesConnections::FromNodeId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesConnections::ToNodeId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesConnections::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PipelineNodesConnections::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes_connections-from_node_id")
                            .from(
                                PipelineNodesConnections::Table,
                                PipelineNodesConnections::FromNodeId,
                            )
                            .to(PipelineNodes::Table, PipelineNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-pipeline_nodes_connections-to_node_id")
                            .from(
                                PipelineNodesConnections::Table,
                                PipelineNodesConnections::ToNodeId,
                            )
                            .to(PipelineNodes::Table, PipelineNodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index 'idx_pipeline_nodes_connections_from_node_id' on 'pipeline_nodes_connections(from_node_id)'
        manager
            .create_index(
                Index::create()
                    .name("idx_pipeline_nodes_connections_from_node_id")
                    .table(PipelineNodesConnections::Table)
                    .col(PipelineNodesConnections::FromNodeId)
                    .to_owned(),
            )
            .await?;

        // Create index 'idx_pipeline_nodes_connections_to_node_id' on 'pipeline_nodes_connections(to_node_id)'
        manager
            .create_index(
                Index::create()
                    .name("idx_pipeline_nodes_connections_to_node_id")
                    .table(PipelineNodesConnections::Table)
                    .col(PipelineNodesConnections::ToNodeId)
                    .to_owned(),
            )
            .await?;

        // Create index 'idx_pipeline_nodes_pipeline_id' on 'pipeline_nodes(pipeline_id)'
        manager
            .create_index(
                Index::create()
                    .name("idx_pipeline_nodes_pipeline_id")
                    .table(PipelineNodes::Table)
                    .col(PipelineNodes::PipelineId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_type(Type::drop().name(Visibility::Enum).if_exists().to_owned())
            .await?;

        // Drop 'exec_status' enum type
        manager
            .drop_type(Type::drop().name(ExecStatus::Enum).if_exists().to_owned())
            .await?;

        // Drop 'node_container_type' enum type
        manager
            .drop_type(
                Type::drop()
                    .name(NodeContainerType::Enum)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        // Drop 'templates' table
        manager
            .drop_table(Table::drop().table(Templates::Table).to_owned())
            .await?;

        // Drop 'pipelines' table
        manager
            .drop_table(Table::drop().table(Pipelines::Table).to_owned())
            .await?;

        // Drop 'pipeline_triggers' table
        manager
            .drop_table(Table::drop().table(PipelineTriggers::Table).to_owned())
            .await?;

        // Drop 'nodes' table
        manager
            .drop_table(Table::drop().table(Nodes::Table).to_owned())
            .await?;

        // Drop 'pipeline_exec' table
        manager
            .drop_table(Table::drop().table(PipelineExec::Table).to_owned())
            .await?;

        // Drop 'pipeline_nodes' table
        manager
            .drop_table(Table::drop().table(PipelineNodes::Table).to_owned())
            .await?;

        // Drop 'pipeline_nodes_exec' table
        manager
            .drop_table(Table::drop().table(PipelineNodesExec::Table).to_owned())
            .await?;

        // Drop index 'idx_pipeline_nodes_pipeline_id'
        manager
            .drop_index(
                Index::drop()
                    .name("idx_pipeline_nodes_pipeline_id")
                    .to_owned(),
            )
            .await?;

        // Drop index 'idx_pipeline_nodes_connections_from_node_id'
        manager
            .drop_index(
                Index::drop()
                    .name("idx_pipeline_nodes_connections_from_node_id")
                    .to_owned(),
            )
            .await?;

        // Drop index 'idx_pipeline_nodes_connections_to_node_id'
        manager
            .drop_index(
                Index::drop()
                    .name("idx_pipeline_nodes_connections_to_node_id")
                    .to_owned(),
            )
            .await?;

        // Drop 'pipeline_nodes_connections' table
        manager
            .drop_table(
                Table::drop()
                    .table(PipelineNodesConnections::Table)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Visibility {
    #[sea_orm(iden = "visibility")]
    Enum,
    #[sea_orm(iden = "public")]
    Public,
    #[sea_orm(iden = "private")]
    Private,
}

#[derive(DeriveIden)]
pub enum ExecStatus {
    #[sea_orm(iden = "exec_status")]
    Enum,
    #[sea_orm(iden = "pending")]
    Pending,
    #[sea_orm(iden = "running")]
    Running,
    #[sea_orm(iden = "completed")]
    Completed,
    #[sea_orm(iden = "failed")]
    Failed,
}

#[derive(DeriveIden)]
pub enum NodeContainerType {
    #[sea_orm(iden = "node_container_type")]
    Enum,
    #[sea_orm(iden = "wasm")]
    Wasm,
    #[sea_orm(iden = "docker")]
    Docker,
}

#[derive(DeriveIden)]
pub enum Templates {
    Table,
    Id,
    Name,
    Description,
    Config,
    Visibility,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Pipelines {
    Table,
    Id,
    Name,
    Description,
    FromTemplateId,
    TeamId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum PipelineTriggers {
    Table,
    Id,
    PipelineId,
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Nodes {
    Table,
    Id,
    Name,
    PublisherName,
    Description,
    Config,
    ContainerType,
    Tags,
    Versions,
    Visibility,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum PipelineExec {
    Table,
    Id,
    PipelineId,
    Status,
    CreatedAt,
    StartedAt,
    FinishedAt,
}

#[derive(DeriveIden)]
pub enum PipelineNodes {
    Table,
    Id,
    PipelineId,
    NodeId,
    NodeVersion,
    TriggerId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum PipelineNodesExec {
    Table,
    Id,
    PipelineExecId,
    PipelineNodeId,
    Status,
    Result,
    CreatedAt,
    StartedAt,
    FinishedAt,
}

#[derive(DeriveIden)]
pub enum PipelineNodesConnections {
    Table,
    Id,
    FromNodeId,
    ToNodeId,
    CreatedAt,
    UpdatedAt,
}
