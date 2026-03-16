pub mod entities;
pub mod seeders;

use crate::infrastructure::database::mysql::seeders::MySQLSeedersRunner;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};

pub struct MySQLDatabase {
    url: String,
}

impl MySQLDatabase {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    pub async fn connect(&self) -> DatabaseConnection {
        tracing::info!("Connecting to mysql database by url: {}", self.url);

        let pool = Database::connect(&self.url)
            .await
            .expect("Database connection failed.");

        tracing::info!("Mysql database successfully connected by url: {}", self.url);

        tracing::info!("Starting migrate database...");

        Migrator::up(&pool, None)
            .await
            .expect("Database migration failed.");

        MySQLSeedersRunner::new(&pool)
            .run()
            .await
            .expect("Failed to seed initialization data to database");

        tracing::info!("Database migrate successfully connected to mysql database");

        pool
    }
}
