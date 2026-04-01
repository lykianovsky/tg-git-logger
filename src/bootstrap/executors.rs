use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::digest::commands::create_digest_subscription::executor::CreateDigestSubscriptionExecutor;
use crate::application::digest::commands::send_due_digests::executor::SendDueDigestsExecutor;
use crate::application::health_ping::commands::check_all_health_pings::executor::CheckAllHealthPingsExecutor;
use crate::application::health_ping::commands::create_health_ping::executor::CreateHealthPingExecutor;
use crate::application::health_ping::commands::delete_health_ping::executor::DeleteHealthPingExecutor;
use crate::application::health_ping::commands::update_health_ping::executor::UpdateHealthPingExecutor;
use crate::application::health_ping::commands::update_health_ping_status::executor::UpdateHealthPingStatusExecutor;
use crate::application::health_ping::queries::get_all_health_pings::executor::GetAllHealthPingsExecutor;
use crate::application::digest::commands::delete_digest_subscription::executor::DeleteDigestSubscriptionExecutor;
use crate::application::digest::commands::toggle_digest_subscription::executor::ToggleDigestSubscriptionExecutor;
use crate::application::digest::commands::update_digest_subscription::executor::UpdateDigestSubscriptionExecutor;
use crate::application::digest::queries::get_user_digest_subscriptions::executor::GetUserDigestSubscriptionsExecutor;
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
use crate::application::user::commands::assign_user_role::executor::AssignUserRoleExecutor;
use crate::application::user::commands::deactivate_user::executor::DeactivateUserExecutor;
use crate::application::user::commands::remove_user_role::executor::RemoveUserRoleExecutor;
use crate::application::user::commands::toggle_user_active::executor::ToggleUserActiveExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::user::commands::unbind_repository::executor::UnbindRepositoryExecutor;
use crate::application::user::queries::get_all_users::executor::GetAllUsersExecutor;
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
    pub get_user_digest_subscriptions: Arc<GetUserDigestSubscriptionsExecutor>,
    pub get_all_health_pings: Arc<GetAllHealthPingsExecutor>,
    pub get_all_users: Arc<GetAllUsersExecutor>,
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
    pub deactivate_user: Arc<DeactivateUserExecutor>,
    pub create_digest_subscription: Arc<CreateDigestSubscriptionExecutor>,
    pub update_digest_subscription: Arc<UpdateDigestSubscriptionExecutor>,
    pub toggle_digest_subscription: Arc<ToggleDigestSubscriptionExecutor>,
    pub delete_digest_subscription: Arc<DeleteDigestSubscriptionExecutor>,
    pub check_all_health_pings: Arc<CheckAllHealthPingsExecutor>,
    pub create_health_ping: Arc<CreateHealthPingExecutor>,
    pub update_health_ping: Arc<UpdateHealthPingExecutor>,
    pub update_health_ping_status: Arc<UpdateHealthPingStatusExecutor>,
    pub delete_health_ping: Arc<DeleteHealthPingExecutor>,

    pub send_due_digests: Arc<SendDueDigestsExecutor>,

    pub toggle_user_active: Arc<ToggleUserActiveExecutor>,
    pub assign_user_role: Arc<AssignUserRoleExecutor>,
    pub remove_user_role: Arc<RemoveUserRoleExecutor>,
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

            get_user_digest_subscriptions: Arc::new(GetUserDigestSubscriptionsExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.digest_subscription_repo.clone(),
            )),

            get_all_health_pings: Arc::new(GetAllHealthPingsExecutor::new(
                shared_dependency.health_ping_repo.clone(),
            )),

            get_all_users: Arc::new(GetAllUsersExecutor::new(
                shared_dependency.user_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_has_roles_repo.clone(),
            )),
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

            deactivate_user: Arc::new(DeactivateUserExecutor::new(
                shared_dependency.user_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
            )),

            create_digest_subscription: Arc::new(CreateDigestSubscriptionExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.digest_subscription_repo.clone(),
            )),

            update_digest_subscription: Arc::new(UpdateDigestSubscriptionExecutor::new(
                shared_dependency.digest_subscription_repo.clone(),
            )),

            toggle_digest_subscription: Arc::new(ToggleDigestSubscriptionExecutor::new(
                shared_dependency.digest_subscription_repo.clone(),
            )),

            delete_digest_subscription: Arc::new(DeleteDigestSubscriptionExecutor::new(
                shared_dependency.digest_subscription_repo.clone(),
            )),

            check_all_health_pings: Arc::new(CheckAllHealthPingsExecutor::new(
                shared_dependency.health_ping_repo.clone(),
                shared_dependency.health_check_client.clone(),
                shared_dependency.notification_service.clone(),
                shared_dependency.user_has_roles_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
            )),

            create_health_ping: Arc::new(CreateHealthPingExecutor::new(
                shared_dependency.health_ping_repo.clone(),
            )),

            update_health_ping: Arc::new(UpdateHealthPingExecutor::new(
                shared_dependency.health_ping_repo.clone(),
            )),

            update_health_ping_status: Arc::new(UpdateHealthPingStatusExecutor::new(
                shared_dependency.health_ping_repo.clone(),
            )),

            delete_health_ping: Arc::new(DeleteHealthPingExecutor::new(
                shared_dependency.health_ping_repo.clone(),
            )),

            send_due_digests: Arc::new(SendDueDigestsExecutor::new(
                shared_dependency.digest_subscription_repo.clone(),
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.notification_service.clone(),
            )),

            toggle_user_active: Arc::new(ToggleUserActiveExecutor::new(
                shared_dependency.user_repo.clone(),
            )),

            assign_user_role: Arc::new(AssignUserRoleExecutor::new(
                mysql_pool.clone(),
                shared_dependency.user_has_roles_repo.clone(),
            )),

            remove_user_role: Arc::new(RemoveUserRoleExecutor::new(
                shared_dependency.user_has_roles_repo.clone(),
            )),
        };

        Self { queries, commands }
    }
}
