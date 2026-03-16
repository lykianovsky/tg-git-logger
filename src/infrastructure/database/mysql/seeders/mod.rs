use crate::infrastructure::database::mysql::seeders::roles::MySQLRolesSeeder;
use crate::infrastructure::database::seeder::Seeder;
use sea_orm::DatabaseConnection;

pub mod roles;

pub struct MySQLSeedersRunner<'a> {
    roles: MySQLRolesSeeder<'a>,
}

impl<'a> MySQLSeedersRunner<'a> {
    pub fn new(db: &'a DatabaseConnection) -> Self {
        Self {
            roles: MySQLRolesSeeder::new(db),
        }
    }

    pub async fn run(self) -> Result<(), String> {
        tracing::debug!("Start seed initialization data to database");

        self.roles.run().await.map_err(|e| {
            tracing::error!("Failed to seed roles to database: {}", e.to_string());
            e.to_string()
        })?;

        Ok(())
    }
}
