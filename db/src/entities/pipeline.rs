use sqlx::types::Uuid;

#[derive(sqlx::FromRow)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: sqlx::types::time::Date,
    pub updated_at: sqlx::types::time::Date,
}
