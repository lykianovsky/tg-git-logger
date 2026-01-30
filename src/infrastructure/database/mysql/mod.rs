pub mod entities;

use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub async fn connect(url: String) -> DatabaseConnection {
    tracing::info!("Connecting to mysql database by url: {url}");

    let db = Database::connect(&url).await.expect("Database connection failed.");

    tracing::info!("Mysql database successfully connected by url: {url}");

    tracing::info!("Starting migrate database...");

    Migrator::up(&db, None).await.expect("Database migration failed.");

    tracing::info!("Database migrate successfully connected to mysql database");

    db
}