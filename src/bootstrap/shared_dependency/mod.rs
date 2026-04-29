use crate::config::application::ApplicationConfig;
use crate::domain::auth::ports::oauth_client::OAuthClient;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::health_ping::ports::health_check_client::HealthCheckClient;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::pending_notification::repositories::pending_notification_repository::PendingNotificationsRepository;
use crate::domain::pr_review::repositories::pr_review_repository::PrReviewRepository;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::repository::repositories::repository_task_tracker_repository::RepositoryTaskTrackerRepository;
use crate::domain::role::repositories::role_repository::RoleRepository;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::user::repositories::user_connection_repositories_repository::UserConnectionRepositoriesRepository;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use crate::domain::user_preferences::services::quiet_hours_resolver::QuietHoursResolver;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::infrastructure::drivers::cache::redis::RedisCache;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::MessageBrokerRabbitMQ;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::MessageBrokerRabbitMQPublisher;
use crate::infrastructure::integrations::health_check::ReqwestHealthCheckClient;
use crate::infrastructure::integrations::oauth::github::GithubOAuthClient;
use crate::infrastructure::integrations::task_tracker::kaiten::{
    KaitenClient, KaitenClientBase, KaitenClientToken,
};
use crate::infrastructure::integrations::version_control::github::client::GithubVersionControlClient;
use crate::infrastructure::processing::event_bus::EventBus;
use crate::infrastructure::repositories::mysql::digest_subscription::MySQLDigestSubscriptionRepository;
use crate::infrastructure::repositories::mysql::health_ping::MySQLHealthPingRepository;
use crate::infrastructure::repositories::mysql::notification_log::MySQLNotificationLogRepository;
use crate::infrastructure::repositories::mysql::pending_notifications::MySQLPendingNotificationsRepository;
use crate::infrastructure::repositories::mysql::pr_review::MySQLPrReviewRepository;
use crate::infrastructure::repositories::mysql::release_plan::MySQLReleasePlanRepository;
use crate::infrastructure::repositories::mysql::repository::MySQLRepositoryRepository;
use crate::infrastructure::repositories::mysql::repository_task_tracker::MySQLRepositoryTaskTrackerRepository;
use crate::infrastructure::repositories::mysql::role::MySQLRoleRepository;
use crate::infrastructure::repositories::mysql::user::MySQLUserRepository;
use crate::infrastructure::repositories::mysql::user_connection_repositories::MySQLUserConnectionRepositoriesRepository;
use crate::infrastructure::repositories::mysql::user_has_roles::MySQLUserHasRolesRepository;
use crate::infrastructure::repositories::mysql::user_preferences::MySQLUserPreferencesRepository;
use crate::infrastructure::repositories::mysql::user_social_accounts::MySQLUserSocialServicesRepository;
use crate::infrastructure::repositories::mysql::user_vc_accounts::MySQLUserVersionControlServicesRepository;
use crate::infrastructure::services::notification::CompositionNotificationService;
use crate::infrastructure::services::task_tracker::kaiten::KaitenTaskTrackerService;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ApplicationSharedDependency {
    pub event_bus: Arc<EventBus>,
    pub message_broker: Arc<dyn MessageBroker>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub reversible_cipher: Arc<ReversibleCipher>,
    pub cache: Arc<dyn CacheService>,
    pub role_repo: Arc<dyn RoleRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_version_controls_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository>,
    pub digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
    pub health_ping_repo: Arc<dyn HealthPingRepository>,
    pub health_check_client: Arc<dyn HealthCheckClient>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    pub notification_service: Arc<CompositionNotificationService>,
    pub oauth_client: Arc<dyn OAuthClient>,
    pub task_tracker_client: Arc<dyn TaskTrackerClient>,
    pub task_tracker_service: Arc<dyn TaskTrackerService>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    pub quiet_hours_resolver: Arc<QuietHoursResolver>,
    pub pending_notifications_repo: Arc<dyn PendingNotificationsRepository>,
    pub pr_review_repo: Arc<dyn PrReviewRepository>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
}

impl ApplicationSharedDependency {
    pub async fn new(
        config: Arc<ApplicationConfig>,
        mysql_pool: Arc<DatabaseConnection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let reversible_cipher = Arc::new(ReversibleCipher::new(
            &config.secret.reversible_cipher_secret,
        ));

        let cache: Arc<dyn CacheService> = Arc::new(
            RedisCache::new(config.redis.url.clone()).expect("Failed to connect to redis"),
        );

        let event_bus = Arc::new(EventBus::new());

        let message_broker =
            Arc::new(MessageBrokerRabbitMQ::new(&config.rabbit_mq.url.clone()).await?);

        let publisher: Arc<dyn MessageBrokerPublisher> =
            Arc::new(MessageBrokerRabbitMQPublisher::new(message_broker.connection.clone()).await?);

        let role_repo = Arc::new(MySQLRoleRepository::new(mysql_pool.clone()));

        let user_repo = Arc::new(MySQLUserRepository::new(mysql_pool.clone()));

        let user_has_roles_repo = Arc::new(MySQLUserHasRolesRepository::new(mysql_pool.clone()));

        let user_socials_repo =
            Arc::new(MySQLUserSocialServicesRepository::new(mysql_pool.clone()));

        let user_version_controls_repo = Arc::new(MySQLUserVersionControlServicesRepository::new(
            mysql_pool.clone(),
        ));

        let user_connection_repositories_repo: Arc<dyn UserConnectionRepositoriesRepository> =
            Arc::new(MySQLUserConnectionRepositoriesRepository::new(
                mysql_pool.clone(),
            ));

        let digest_subscription_repo: Arc<dyn DigestSubscriptionRepository> =
            Arc::new(MySQLDigestSubscriptionRepository::new(mysql_pool.clone()));

        let health_ping_repo: Arc<dyn HealthPingRepository> =
            Arc::new(MySQLHealthPingRepository::new(mysql_pool.clone()));

        let health_check_client: Arc<dyn HealthCheckClient> =
            Arc::new(ReqwestHealthCheckClient::new());

        let repository_repo: Arc<dyn RepositoryRepository> =
            Arc::new(MySQLRepositoryRepository::new(mysql_pool.clone()));

        let repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository> = Arc::new(
            MySQLRepositoryTaskTrackerRepository::new(mysql_pool.clone()),
        );

        let notification_service = Arc::new(CompositionNotificationService::new(
            config.telegram.bot_token.clone(),
        ));

        let oauth_client: Arc<dyn OAuthClient> = Arc::new(GithubOAuthClient::new(
            config.github.base.clone(),
            config.github.oauth_client_id.clone(),
            config.github.oauth_client_secret.clone(),
        ));

        let task_tracker_client: Arc<dyn TaskTrackerClient> = Arc::new(KaitenClient::new(
            KaitenClientBase(config.kaiten.base.clone()),
            KaitenClientToken(config.kaiten.api_token.clone()),
        ));

        let task_tracker_service: Arc<dyn TaskTrackerService> = Arc::new(
            KaitenTaskTrackerService::new(config.task_tracker.extract_pattern.clone()),
        );

        let version_control_client: Arc<dyn VersionControlClient> = Arc::new(
            GithubVersionControlClient::new(config.github.api_base.clone()),
        );

        let user_preferences_repo: Arc<dyn UserPreferencesRepository> =
            Arc::new(MySQLUserPreferencesRepository::new(mysql_pool.clone()));

        let default_quiet_hours_window = QuietHoursWindow::new(
            config.notifications.default_dnd_start,
            config.notifications.default_dnd_end,
        );
        let quiet_hours_resolver = Arc::new(QuietHoursResolver::new(
            default_quiet_hours_window,
            config.notifications.default_timezone,
        ));

        let pending_notifications_repo: Arc<dyn PendingNotificationsRepository> =
            Arc::new(MySQLPendingNotificationsRepository::new(mysql_pool.clone()));

        let pr_review_repo: Arc<dyn PrReviewRepository> =
            Arc::new(MySQLPrReviewRepository::new(mysql_pool.clone()));

        let notification_log_repo: Arc<dyn NotificationLogRepository> =
            Arc::new(MySQLNotificationLogRepository::new(mysql_pool.clone()));

        let release_plan_repo: Arc<dyn ReleasePlanRepository> =
            Arc::new(MySQLReleasePlanRepository::new(mysql_pool.clone()));

        Ok(Self {
            event_bus,
            message_broker,
            publisher,
            reversible_cipher,
            cache,
            role_repo,
            user_repo,
            user_has_roles_repo,
            user_socials_repo,
            user_version_controls_repo,
            user_connection_repositories_repo,
            digest_subscription_repo,
            health_ping_repo,
            health_check_client,
            repository_repo,
            repository_task_tracker_repo,
            notification_service,
            oauth_client,
            task_tracker_client,
            task_tracker_service,
            version_control_client,
            user_preferences_repo,
            quiet_hours_resolver,
            pending_notifications_repo,
            pr_review_repo,
            notification_log_repo,
            release_plan_repo,
        })
    }
}
