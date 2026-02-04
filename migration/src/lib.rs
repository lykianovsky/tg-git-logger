pub use sea_orm_migration::prelude::*;

mod m001_create_users;
mod m002_create_roles;
mod m003_create_user_roles;
mod m004_create_github_accounts;
mod m005_create_telegram_accounts;
mod m006_create_repositories;
mod m007_create_repository_users;
mod m008_create_repository_settings;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_users::Migration),
            Box::new(m002_create_roles::Migration),
            Box::new(m003_create_user_roles::Migration),
            Box::new(m004_create_github_accounts::Migration),
            Box::new(m005_create_telegram_accounts::Migration),
            Box::new(m006_create_repositories::Migration),
            Box::new(m007_create_repository_users::Migration),
            Box::new(m008_create_repository_settings::Migration),
        ]
    }
}
