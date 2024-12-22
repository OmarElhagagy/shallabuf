pub mod dtos;

#[cfg(feature = "seed")]
pub mod seed;

use sqlx::migrate::Migrator;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
