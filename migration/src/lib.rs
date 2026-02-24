pub use sea_orm_migration::prelude::*;

mod m001_create_users;
mod m002_create_roles;
mod m003_create_user_has_role;
mod m004_create_user_social_accounts;
mod m005_create_user_version_control_accounts;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m001_create_users::Migration),
            Box::new(m002_create_roles::Migration),
            Box::new(m003_create_user_has_role::Migration),
            Box::new(m004_create_user_social_accounts::Migration),
            Box::new(m005_create_user_version_control_accounts::Migration),
        ]
    }
}
