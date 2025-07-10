use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::infrastructure::database::database::{DatabaseError, DatabaseOptions};

#[non_exhaustive]
pub struct PostgresDatabase {
    pool: PgPool,
}

impl PostgresDatabase {
    pub async fn connect(options: DatabaseOptions) -> Result<Self, DatabaseError> {
        // Get postgres configuration.
        let connection_url = options.postgres.connection_url();
        let max_connections = options.postgres.max_connections();

        // Connect to the database and get a connection pool.
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&connection_url)
            .await?;

        tracing::info!("Connected to PostgreSQL database.");

        Ok(Self { pool })
    }

    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}
