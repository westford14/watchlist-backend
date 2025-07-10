use sqlx::{PgConnection, PgPool};
use thiserror::Error;

use crate::infrastructure::database::postgres::PostgresDatabase;
use crate::infrastructure::database::postgres::PostgresOptions;

pub type DatabasePool = PgPool;
pub type DatabaseConnection = PgConnection;
pub type TestDatabase = PostgresDatabase;

#[derive(Clone, Debug)]
pub struct DatabaseOptions {
    pub postgres: PostgresOptions,
}

pub struct Database;

impl Database {
    pub async fn connect(options: DatabaseOptions) -> Result<DatabasePool, DatabaseError> {
        let db = PostgresDatabase::connect(options).await?;
        Ok(db.pool().clone())
    }
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error(transparent)]
    SQLxError(#[from] sqlx::Error),
    #[error(transparent)]
    SQLxMigrateError(#[from] sqlx::migrate::MigrateError),
}
