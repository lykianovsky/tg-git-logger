use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::notification::commands::send_social_notify::executor::SendSocialNotifyExecutor;
use crate::application::task::commands::move_task_to_test::executor::MoveTaskToTestExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::utils::mutex::key_locker::KeyLocker;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ApplicationBoostrapExecutorsQueries {
    pub build_report_by_range: Arc<BuildVersionControlDateRangeReportExecutor>,
}

pub struct ApplicationBoostrapExecutorsCommands {
    pub register_user_via_oauth: Arc<RegisterUserViaOAuthExecutor>,
    pub create_oauth_link: Arc<CreateOAuthLinkExecutor>,
    pub dispatch_webhook_event: Arc<DispatchWebhookEventExecutor>,
    pub send_social_notify: Arc<SendSocialNotifyExecutor>,
    pub move_task_to_test: Arc<MoveTaskToTestExecutor>,
}

pub struct ApplicationBoostrapExecutors {
    pub queries: ApplicationBoostrapExecutorsQueries,
    pub commands: ApplicationBoostrapExecutorsCommands,
}

impl ApplicationBoostrapExecutors {
    pub fn new(
        config: Arc<ApplicationConfig>,
        mysql_pool: Arc<DatabaseConnection>,
        shared_dependency: Arc<ApplicationSharedDependency>,
    ) -> Self {
        let queries = ApplicationBoostrapExecutorsQueries {
            build_report_by_range: Arc::new(BuildVersionControlDateRangeReportExecutor {
                reversible_cipher: shared_dependency.reversible_cipher.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                user_version_control_service_repo: shared_dependency
                    .user_version_controls_repo
                    .clone(),
                version_control_client: shared_dependency.version_control_client.clone(),
            }),
        };

        let commands = ApplicationBoostrapExecutorsCommands {
            create_oauth_link: Arc::new(CreateOAuthLinkExecutor::new(
                shared_dependency.user_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.cache.clone(),
            )),
            register_user_via_oauth: Arc::new(RegisterUserViaOAuthExecutor {
                db: mysql_pool.clone(),
                user_repo: shared_dependency.user_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                user_version_control_service_repo: shared_dependency
                    .user_version_controls_repo
                    .clone(),
                oauth_client: shared_dependency.oauth_client.clone(),
                version_control_client: shared_dependency.version_control_client.clone(),
                reversible_cipher: shared_dependency.reversible_cipher.clone(),
                notification_service: shared_dependency.notification_service.clone(),
                cache: shared_dependency.cache.clone(),
                mutex: Arc::new(KeyLocker::new()),
            }),
            dispatch_webhook_event: Arc::new(DispatchWebhookEventExecutor {
                publisher: shared_dependency.publisher.clone(),
            }),
            send_social_notify: Arc::new(SendSocialNotifyExecutor::new(
                shared_dependency.notification_service.clone(),
            )),
            move_task_to_test: Arc::new(MoveTaskToTestExecutor::new(
                shared_dependency.task_tracker_client.clone(),
                shared_dependency.task_tracker_service.clone(),
                config.task_tracker.test_column_id,
            )),
        };

        Self { queries, commands }
    }
}
