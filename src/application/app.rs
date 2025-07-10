use std::sync::Arc;

use crate::{
    api::server,
    application::{config, state::AppState},
    infrastructure::{database::database::Database, redis},
};

pub async fn run() {
    // Load configuration.
    let config = config::load();

    // Connect to PostgreSQL.
    let db_pool = Database::connect(config.clone().into())
        .await
        .expect("Failed to connect to the database.");

    // Connect to Redis.
    let redis = redis::open(&config).await.into();

    // Build the application state.
    let shared_state = Arc::new(AppState {
        config,
        db_pool,
        redis,
    });

    server::start(shared_state).await;
}
