pub mod dtos;
pub mod entities;
mod migrations;
pub mod seed;

pub use migrations::*;
pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_auth_schema::Migration),
            Box::new(m20241208_130842_pipeline_schema::Migration),
        ]
    }
}
