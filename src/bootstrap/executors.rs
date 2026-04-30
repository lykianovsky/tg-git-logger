use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::digest::commands::create_digest_subscription::executor::CreateDigestSubscriptionExecutor;
use crate::application::digest::commands::delete_digest_subscription::executor::DeleteDigestSubscriptionExecutor;
use crate::application::digest::commands::send_due_digests::executor::SendDueDigestsExecutor;
use crate::application::digest::commands::toggle_digest_subscription::executor::ToggleDigestSubscriptionExecutor;
use crate::application::digest::commands::update_digest_subscription::executor::UpdateDigestSubscriptionExecutor;
use crate::application::digest::queries::get_user_digest_subscriptions::executor::GetUserDigestSubscriptionsExecutor;
use crate::application::health_ping::commands::check_all_health_pings::executor::CheckAllHealthPingsExecutor;
use crate::application::health_ping::commands::create_health_ping::executor::CreateHealthPingExecutor;
use crate::application::health_ping::commands::delete_health_ping::executor::DeleteHealthPingExecutor;
use crate::application::health_ping::commands::update_health_ping::executor::UpdateHealthPingExecutor;
use crate::application::health_ping::commands::update_health_ping_status::executor::UpdateHealthPingStatusExecutor;
use crate::application::health_ping::queries::get_all_health_pings::executor::GetAllHealthPingsExecutor;
use crate::application::monitoring::queries::get_queues_stats::executor::GetQueuesStatsExecutor;
use crate::application::notification::commands::buffer_notification::executor::BufferNotificationExecutor;
use crate::application::notification::commands::flush_pending_notifications::executor::FlushPendingNotificationsExecutor;
use crate::application::notification::commands::scan_pr_conflicts::executor::ScanPrConflictsExecutor;
use crate::application::notification::commands::scan_stale_pull_requests::executor::ScanStalePullRequestsExecutor;
use crate::application::notification::commands::send_social_notify::executor::SendSocialNotifyExecutor;
use crate::application::release_plan::commands::cancel_release_plan::executor::CancelReleasePlanExecutor;
use crate::application::release_plan::commands::complete_release_plan::executor::CompleteReleasePlanExecutor;
use crate::application::release_plan::commands::create_release_plan::executor::CreateReleasePlanExecutor;
use crate::application::release_plan::commands::send_call_reminders::executor::SendCallRemindersExecutor;
use crate::application::release_plan::commands::send_release_day_reminders::executor::SendReleaseDayRemindersExecutor;
use crate::application::release_plan::commands::update_release_plan::executor::UpdateReleasePlanExecutor;
use crate::application::release_plan::queries::get_upcoming_release_plans::executor::GetUpcomingReleasePlansExecutor;
use crate::application::repository::commands::create_repository::executor::CreateRepositoryExecutor;
use crate::application::repository::commands::create_repository_task_tracker::executor::CreateRepositoryTaskTrackerExecutor;
use crate::application::repository::commands::delete_repository::executor::DeleteRepositoryExecutor;
use crate::application::repository::commands::set_repository_notification_chat::executor::SetRepositoryNotificationChatExecutor;
use crate::application::repository::commands::set_repository_notifications_chat::executor::SetRepositoryNotificationsChatExecutor;
use crate::application::repository::commands::unset_repository_notification_chat::executor::UnsetRepositoryNotificationChatExecutor;
use crate::application::repository::commands::update_repository::executor::UpdateRepositoryExecutor;
use crate::application::repository::commands::update_repository_task_tracker::executor::UpdateRepositoryTaskTrackerExecutor;
use crate::application::repository::queries::get_all_repositories::executor::GetAllRepositoriesExecutor;
use crate::application::task::commands::move_task_to_test::executor::MoveTaskToTestExecutor;
use crate::application::task::queries::get_task_card::executor::GetTaskCardExecutor;
use crate::application::user::commands::assign_user_role::executor::AssignUserRoleExecutor;
use crate::application::user::commands::bind_repository::executor::BindRepositoryExecutor;
use crate::application::user::commands::deactivate_user::executor::DeactivateUserExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::user::commands::remove_user_role::executor::RemoveUserRoleExecutor;
use crate::application::user::commands::toggle_user_active::executor::ToggleUserActiveExecutor;
use crate::application::user::commands::unbind_repository::executor::UnbindRepositoryExecutor;
use crate::application::user::queries::get_all_users::executor::GetAllUsersExecutor;
use crate::application::user::queries::get_my_pull_requests::executor::GetMyPullRequestsExecutor;
use crate::application::user::queries::get_pending_reviews::executor::GetPendingReviewsExecutor;
use crate::application::user::queries::get_user_bound_repositories::executor::GetUserBoundRepositoriesExecutor;
use crate::application::user::queries::get_user_overview::executor::GetUserOverviewExecutor;
use crate::application::user::queries::get_user_roles_by_telegram_id::executor::GetUserRolesByTelegramIdExecutor;
use crate::application::user_preferences::commands::update_user_preferences::executor::UpdateUserPreferencesExecutor;
use crate::application::user_preferences::queries::get_user_preferences::executor::GetUserPreferencesExecutor;
use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::domain::monitoring::ports::workers_stats_provider::WorkersStatsProvider;
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
    pub get_user_preferences: Arc<GetUserPreferencesExecutor>,
    pub get_upcoming_release_plans: Arc<GetUpcomingReleasePlansExecutor>,
    pub get_user_overview: Arc<GetUserOverviewExecutor>,
    pub get_my_pull_requests: Arc<GetMyPullRequestsExecutor>,
    pub get_pending_reviews: Arc<GetPendingReviewsExecutor>,
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
    pub set_repository_notifications_chat: Arc<SetRepositoryNotificationsChatExecutor>,
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

    pub flush_pending_notifications: Arc<FlushPendingNotificationsExecutor>,

    pub update_user_preferences: Arc<UpdateUserPreferencesExecutor>,

    pub scan_stale_pull_requests: Arc<ScanStalePullRequestsExecutor>,
    pub scan_pr_conflicts: Arc<ScanPrConflictsExecutor>,

    pub create_release_plan: Arc<CreateReleasePlanExecutor>,
    pub update_release_plan: Arc<UpdateReleasePlanExecutor>,
    pub cancel_release_plan: Arc<CancelReleasePlanExecutor>,
    pub complete_release_plan: Arc<CompleteReleasePlanExecutor>,
    pub send_release_day_reminders: Arc<SendReleaseDayRemindersExecutor>,
    pub send_call_reminders: Arc<SendCallRemindersExecutor>,
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

            get_user_preferences: Arc::new(GetUserPreferencesExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_preferences_repo.clone(),
            )),

            get_upcoming_release_plans: Arc::new(GetUpcomingReleasePlansExecutor::new(
                shared_dependency.release_plan_repo.clone(),
            )),

            get_user_overview: Arc::new(GetUserOverviewExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_has_roles_repo.clone(),
                shared_dependency.user_version_controls_repo.clone(),
                shared_dependency.user_preferences_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
                shared_dependency.repository_repo.clone(),
            )),

            get_my_pull_requests: Arc::new(GetMyPullRequestsExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_version_controls_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
                shared_dependency.repository_repo.clone(),
                shared_dependency.version_control_client.clone(),
                shared_dependency.reversible_cipher.clone(),
            )),

            get_pending_reviews: Arc::new(GetPendingReviewsExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_version_controls_repo.clone(),
                shared_dependency.user_connection_repositories_repo.clone(),
                shared_dependency.repository_repo.clone(),
                shared_dependency.version_control_client.clone(),
                shared_dependency.reversible_cipher.clone(),
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
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_preferences_repo.clone(),
                shared_dependency.quiet_hours_resolver.clone(),
                Arc::new(BufferNotificationExecutor::new(
                    shared_dependency.pending_notifications_repo.clone(),
                )),
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
            set_repository_notifications_chat: Arc::new(
                SetRepositoryNotificationsChatExecutor::new(
                    mysql_pool.clone(),
                    shared_dependency.repository_repo.clone(),
                ),
            ),
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

            flush_pending_notifications: Arc::new(FlushPendingNotificationsExecutor::new(
                shared_dependency.pending_notifications_repo.clone(),
                shared_dependency.notification_service.clone(),
            )),

            update_user_preferences: Arc::new(UpdateUserPreferencesExecutor::new(
                shared_dependency.user_socials_repo.clone(),
                shared_dependency.user_preferences_repo.clone(),
            )),

            scan_stale_pull_requests: Arc::new(ScanStalePullRequestsExecutor {
                publisher: shared_dependency.publisher.clone(),
                repository_repo: shared_dependency.repository_repo.clone(),
                pr_review_repo: shared_dependency.pr_review_repo.clone(),
                user_has_roles_repo: shared_dependency.user_has_roles_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                user_vc_accounts_repo: shared_dependency.user_version_controls_repo.clone(),
                version_control_client: shared_dependency.version_control_client.clone(),
                reversible_cipher: shared_dependency.reversible_cipher.clone(),
                stale_threshold_hours: config.notifications.stale_threshold_hours,
            }),

            scan_pr_conflicts: Arc::new(ScanPrConflictsExecutor {
                publisher: shared_dependency.publisher.clone(),
                repository_repo: shared_dependency.repository_repo.clone(),
                notification_log_repo: shared_dependency.notification_log_repo.clone(),
                user_has_roles_repo: shared_dependency.user_has_roles_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                user_vc_accounts_repo: shared_dependency.user_version_controls_repo.clone(),
                version_control_client: shared_dependency.version_control_client.clone(),
                reversible_cipher: shared_dependency.reversible_cipher.clone(),
            }),

            create_release_plan: Arc::new(CreateReleasePlanExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                repository_repo: shared_dependency.repository_repo.clone(),
                publisher: shared_dependency.publisher.clone(),
            }),

            update_release_plan: Arc::new(UpdateReleasePlanExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
            }),

            cancel_release_plan: Arc::new(CancelReleasePlanExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
                user_socials_repo: shared_dependency.user_socials_repo.clone(),
                publisher: shared_dependency.publisher.clone(),
            }),

            complete_release_plan: Arc::new(CompleteReleasePlanExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
            }),

            send_release_day_reminders: Arc::new(SendReleaseDayRemindersExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
                repository_repo: shared_dependency.repository_repo.clone(),
                publisher: shared_dependency.publisher.clone(),
            }),

            send_call_reminders: Arc::new(SendCallRemindersExecutor {
                release_plan_repo: shared_dependency.release_plan_repo.clone(),
                publisher: shared_dependency.publisher.clone(),
            }),
        };

        Self { queries, commands }
    }
}
