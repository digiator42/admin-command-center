use gritshield::core::env::get_env;
use sea_orm::DatabaseConnection;
use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn connect_and_migrate_db() -> DatabaseConnection {
    let url = get_env("PG_URL", "");

    let sqlx_pool = sqlx::PgPool::connect(&url)
        .await
        .unwrap();

    println!("[DB] Checking database migration schema integrity via embedded assets...");

    MIGRATOR
        .run(&sqlx_pool)
        .await
        .expect("Failed to execute embedded database schema migrations");

    sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(sqlx_pool)
}
