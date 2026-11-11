pub mod dashboard;
pub mod migration;
pub mod models;
pub mod services;
pub mod settings;
pub mod users;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("database connection failed: {0}")]
    Connection(#[from] sea_orm::DbErr),
    #[error("database operation failed: {0}")]
    Operation(String),
}

pub async fn connect_database(url: &str) -> Result<DatabaseConnection, InfrastructureError> {
    let mut options = ConnectOptions::new(url.to_owned());
    options
        .max_connections(8)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);
    Ok(Database::connect(options).await?)
}

pub async fn run_migrations(database: &DatabaseConnection) -> Result<(), InfrastructureError> {
    use sea_orm_migration::MigratorTrait;
    migration::Migrator::up(database, None)
        .await
        .map_err(InfrastructureError::Connection)?;
    Ok(())
}

pub struct RuntimeStatus {
    pub database_ready: bool,
    pub storage_ready: bool,
}

impl RuntimeStatus {
    pub fn ready() -> Self {
        Self {
            database_ready: true,
            storage_ready: true,
        }
    }
}
