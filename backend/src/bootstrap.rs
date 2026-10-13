use std::sync::Arc;
use std::time::Duration;

use gritshield::core::env::{get_env, initialize_env};
use gritshield::database::{DbConfig, DbManager};
use sea_orm::DatabaseConnection;

/// Connects through GritShield's built-in [`DbManager`], which also applies any
/// pending `*.sql` files found in `./migrations` (framework migration ledger).
///
/// The DSN is read from `DATABASE_URL` (falling back to `PG_URL`); when neither
/// is set the framework transparently falls back to an in-memory SQLite link.
pub async fn connect_and_migrate_db() -> Arc<DatabaseConnection> {
    initialize_env();

    let url = get_env("DATABASE_URL", &get_env("PG_URL", ""));

    let config = DbConfig {
        url,
        max_connections: 20,
        min_connections: 5,
        connect_timeout: Duration::from_secs(10),
        idle_timeout: Duration::from_secs(300),
    };

    DbManager::connect(config)
        .await
        .expect("failed to connect to the database and apply migrations")
}
