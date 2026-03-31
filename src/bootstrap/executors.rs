use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::monitoring::queries::get_queues_stats::executor::GetQueuesStatsExecutor;
use crate::domain::monitoring::ports::workers_stats_provider::WorkersStatsProvider;
use crate::application::notification::commands::send_social_notify::executor::SendSocialNotifyExecutor;
use crate::application::repository::commands::create_repository::executor::CreateRepositoryExecutor;
use crate::application::repository::commands::create_repository_task_tracker::executor::CreateRepositoryTaskTrackerExecutor;
use crate::application::repository::commands::delete_repository::executor::DeleteRepositoryExecutor;
use crate::application::repository::commands::set_repository_notification_chat::executor::SetRepositoryNotificationChatExecutor;
use crate::application::repository::commands::unset_repository_notification_chat::executor::UnsetRepositoryNotificationChatExecutor;
use crate::application::repository::commands::update_repository::executor::UpdateRepositoryExecutor;
use crate::application::repository::commands::update_repository_task_tracker::executor::UpdateRepositoryTaskTrackerExecutor;
use crate::application::repository::queries::get_all_repositories::executor::GetAllRepositoriesExecutor;
use crate::application::task::commands::move_task_to_test::executor::MoveTaskToTestExecutor;
use crate::application::task::queries::get_task_card::executor::GetTaskCardExecutor;
use crate::application::user::commands::bind_repository::executor::BindRepositoryExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::user::commands::unbind_repository::executor::UnbindRepositoryExecutor;
use crate::application::user::queries::get_user_bound_repositories::executor::GetUserBoundRepositoriesExecutor;
use crate::application::user::queries::get_user_roles_by_telegram_id::executor::GetUserRolesByTelegramIdExecutor;
use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::utils::mutex::key_locker::KeyLocker;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ApplicationBoostrapExecutorsQueries {
    pub build_report_by_range: Arc<BuildVersionControlDateRangeReportExecutor>,
    pub get_user_roles_by_telegram_id: Arc<GetUserRolesByTelegramIdExecutor>,
    pub get_user_bound_repositories: Arc<GetUserBoundRepositoriesExecutor>,
    pub get_all_repositories: Arc<GetAllRepositoriesExecutor>,
    pub get_task_card: Arc<GetTaskCardExecutor>,
    pub get_queues_stats: Arc<GetQueuesStatsExecutor>,
}

pub struct ApplicationBoostrapExecutorsCommands {
    pub register_user_via_oauth: Arc<RegisterUserViaOAuthExecutor>,
    pub create_oauth_link: Arc<CreateOAuthLinkExecutor>,
    pub dispatch_webhook_event: Arc<DispatchWebhookEventExecutor>,
    pub send_social_notify: Arc<SendSocialNotifyExecutor>,
    pub move_task_to_test: Arc<MoveTaskToTestExecutor>,
    pub create_repository: Arc<CreateRepositoryExecutor>,
    pub create_repository_task_tracker: Arc<CreateRepositoryTaskTrackerExecutor>,
    pub update_repository: Arc<UpdateRepositoryExecutor>,
    pub update_repository_task_tracker: Arc<UpdateRepositoryTaskTrackerExecutor>,
    pub set_repository_notification_chat: Arc<SetRepositoryNotificationChatExecutor>,
    pub unset_repository_notification_chat: Arc<UnsetRepositoryNotificationChatExecutor>,
    pub bind_repository: Arc<BindRepositoryExecutor>,
    pub unbind_repository: Arc<UnbindRepositoryExecutor>,
    pub delete_repository: Arc<DeleteRepositoryExecutor>,
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
        stats_provider: Arc<dyn WorkersStatsProvider>,
    ) -> Self {
        let queries = ApplicationBoostrapExecutorsQueries {
            build_report_by_range: Arc::new(BuildVersionControlDateRangeReportExecutor::new(
                shared_dependency.reversible_cipher.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_version_controls_repo.clone(),
                shared_dependency.version_control_client.clone(),
                shared_dependency.repository_repo.clone(),
                shared_dependency.repository_task_tracker_repo.clone(),
                shared_dependency.task_tracker_service.clone(),
                config.kaiten.base.clone(),
                config.base_url.clone(),
                shared_dependency.cache.clone(),
            )),
            get_user_roles_by_telegram_id: Arc::new(GetUserRolesByTelegramIdExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_has_roles_repo.clone(),
            )),
            get_user_bound_repositories: Arc::new(GetUserBoundRepositoriesExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
                shared_dependency.repository_repo.clone(),
            )),
            get_all_repositories: Arc::new(GetAllRepositoriesExecutor::new(
                shared_dependency.repository_repo.clone(),
            )),
            get_task_card: Arc::new(GetTaskCardExecutor::new(
                shared_dependency.task_tracker_client.clone(),
            )),
            get_queues_stats: Arc::new(GetQueuesStatsExecutor { stats_provider }),
        };

        let commands = ApplicationBoostrapExecutorsCommands {
            create_oauth_link: Arc::new(CreateOAuthLinkExecutor::new(
                shared_dependency.user_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.cache.clone(),
            )),
            register_user_via_oauth: Arc::new(RegisterUserViaOAuthExecutor {
                db: mysql_pool.clone(),
                user_has_role: shared_dependency.user_has_roles_repo.clone(),
                user_repo: shared_dependency.user_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                user_version_control_service_repo: shared_dependency
                    .user_version_controls_repo
                    .clone(),
                oauth_client: shared_dependency.oauth_client.clone(),
                version_control_client: shared_dependency.version_control_client.clone(),
                reversible_cipher: shared_dependency.reversible_cipher.clone(),
                cache: shared_dependency.cache.clone(),
                mutex: Arc::new(KeyLocker::new()),
                telegram_admin_user_id: SocialUserId(config.telegram.admin_user_id as i32),
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
            create_repository: Arc::new(CreateRepositoryExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_repo.clone(),
            )),
            create_repository_task_tracker: Arc::new(CreateRepositoryTaskTrackerExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_task_tracker_repo.clone(),
            )),
            update_repository: Arc::new(UpdateRepositoryExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_repo.clone(),
            )),
            update_repository_task_tracker: Arc::new(UpdateRepositoryTaskTrackerExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_task_tracker_repo.clone(),
            )),
            set_repository_notification_chat: Arc::new(SetRepositoryNotificationChatExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_repo.clone(),
            )),
            unset_repository_notification_chat: Arc::new(
                UnsetRepositoryNotificationChatExecutor::new(
                    mysql_pool.clone(),
                    shared_dependency.repository_repo.clone(),
                ),
            ),
            bind_repository: Arc::new(BindRepositoryExecutor::new(
                mysql_pool.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.repository_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
            )),
            unbind_repository: Arc::new(UnbindRepositoryExecutor::new(
                mysql_pool.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
            )),
            delete_repository: Arc::new(DeleteRepositoryExecutor::new(
                mysql_pool.clone(),
                shared_dependency.repository_repo.clone(),
            )),
        };

        Self { queries, commands }
    }
}
