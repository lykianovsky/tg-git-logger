pub use sea_orm_migration::prelude::*;

mod m001_create_users;
mod m002_create_roles;
mod m003_create_user_has_role;
mod m004_create_user_social_accounts;
mod m005_create_user_version_control_accounts;
mod m006_create_repositories;
mod m007_create_repository_pull_request;
mod m008_create_user_connection_repositories;
mod m009_create_user_notifications;
mod m20260326_144331_create_repository_task_trackers;

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
            Box::new(m006_create_repositories::Migration),
            Box::new(m007_create_repository_pull_request::Migration),
            Box::new(m008_create_user_connection_repositories::Migration),
            Box::new(m009_create_user_notifications::Migration),
            Box::new(m20260326_144331_create_repository_task_trackers::Migration),
        ]
    }
}
