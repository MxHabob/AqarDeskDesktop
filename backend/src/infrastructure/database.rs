use sea_orm::{ConnectOptions, DatabaseConnection};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection could not be established.")]
    Connect(#[source] sea_orm::DbErr),
}

pub struct Database {
    connection: DatabaseConnection,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, DatabaseError> {
        let mut options = ConnectOptions::new(url.to_owned());
        let is_sqlite = url.starts_with("sqlite:");
        options
            .max_connections(if is_sqlite { 1 } else { 8 })
            .min_connections(1)
            .connect_timeout(Duration::from_secs(10))
            .acquire_timeout(Duration::from_secs(10))
            .sqlx_logging(false);

        let connection = sea_orm::Database::connect(options)
            .await
            .map_err(DatabaseError::Connect)?;
        Ok(Self { connection })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub async fn ping(&self) -> Result<(), DatabaseError> {
        self.connection.ping().await.map_err(DatabaseError::Connect)
    }

    pub async fn close(self) -> Result<(), DatabaseError> {
        self.connection
            .close()
            .await
            .map_err(DatabaseError::Connect)
    }
}
