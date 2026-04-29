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
mod m20260327_000001_add_telegram_chat_id_to_repositories;
mod m20260328_000001_rename_telegram_chat_id_to_social_chat_id_in_repositories;
mod m20260328_000002_add_unique_user_connection_repositories;
mod m20260401_000001_create_digest_subscriptions;
mod m20260401_000002_create_health_pings;
mod m20260401_000003_add_failed_since_to_health_pings;
mod m20260429_000001_create_user_preferences;
mod m20260429_000002_create_pending_notifications;
mod m20260429_000003_create_pr_reviews;
mod m20260429_000004_create_notification_log;
mod m20260429_000005_create_release_plans;
mod m20260429_000006_create_release_plan_repositories;
mod m20260429_000007_add_notifications_chat_id_to_repositories;
mod m20260429_000008_pending_notifications_user_id_nullable;

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
            Box::new(m20260327_000001_add_telegram_chat_id_to_repositories::Migration),
            Box::new(
                m20260328_000001_rename_telegram_chat_id_to_social_chat_id_in_repositories::Migration,
            ),
            Box::new(
                m20260328_000002_add_unique_user_connection_repositories::Migration,
            ),
            Box::new(m20260401_000001_create_digest_subscriptions::Migration),
            Box::new(m20260401_000002_create_health_pings::Migration),
            Box::new(m20260401_000003_add_failed_since_to_health_pings::Migration),
            Box::new(m20260429_000001_create_user_preferences::Migration),
            Box::new(m20260429_000002_create_pending_notifications::Migration),
            Box::new(m20260429_000003_create_pr_reviews::Migration),
            Box::new(m20260429_000004_create_notification_log::Migration),
            Box::new(m20260429_000005_create_release_plans::Migration),
            Box::new(m20260429_000006_create_release_plan_repositories::Migration),
            Box::new(m20260429_000007_add_notifications_chat_id_to_repositories::Migration),
            Box::new(m20260429_000008_pending_notifications_user_id_nullable::Migration),
        ]
    }
}
