use crate::config::application::ApplicationConfig;
use crate::domain::auth::ports::oauth_client::OAuthClient;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::infrastructure::drivers::cache::redis::RedisCache;
use crate::infrastructure::drivers::message_broker::contracts::broker::MessageBroker;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::infrastructure::drivers::message_broker::rabbitmq::broker::MessageBrokerRabbitMQ;
use crate::infrastructure::drivers::message_broker::rabbitmq::publisher::MessageBrokerRabbitMQPublisher;
use crate::infrastructure::integrations::oauth::github::GithubOAuthClient;
use crate::infrastructure::integrations::task_tracker::kaiten::{
    KaitenClient, KaitenClientBase, KaitenClientToken,
};
use crate::infrastructure::integrations::version_control::github::client::GithubVersionControlClient;
use crate::infrastructure::processing::event_bus::EventBus;
use crate::infrastructure::repositories::mysql::user::MySQLUserRepository;
use crate::infrastructure::repositories::mysql::user_has_roles::MySQLUserHasRolesRepository;
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
    pub user_repo: Arc<MySQLUserRepository>,
    pub user_has_roles_repo: Arc<MySQLUserHasRolesRepository>,
    pub user_socials_repo: Arc<MySQLUserSocialServicesRepository>,
    pub user_version_controls_repo: Arc<MySQLUserVersionControlServicesRepository>,
    pub notification_service: Arc<CompositionNotificationService>,
    pub oauth_client: Arc<dyn OAuthClient>,
    pub task_tracker_client: Arc<dyn TaskTrackerClient>,
    pub task_tracker_service: Arc<dyn TaskTrackerService>,
    pub version_control_client: Arc<dyn VersionControlClient>,
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

        let user_repo = Arc::new(MySQLUserRepository::new(mysql_pool.clone()));

        let user_has_roles_repo = Arc::new(MySQLUserHasRolesRepository::new(mysql_pool.clone()));

        let user_socials_repo =
            Arc::new(MySQLUserSocialServicesRepository::new(mysql_pool.clone()));

        let user_version_controls_repo = Arc::new(MySQLUserVersionControlServicesRepository::new(
            mysql_pool.clone(),
        ));

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

        let version_control_client: Arc<dyn VersionControlClient> =
            Arc::new(GithubVersionControlClient::new(
                config.github.api_base.clone(),
                config.github.repository_owner.clone(),
                config.github.repository.clone(),
            ));

        Ok(Self {
            event_bus,
            message_broker,
            publisher,
            reversible_cipher,
            cache,
            user_repo,
            user_has_roles_repo,
            user_socials_repo,
            user_version_controls_repo,
            notification_service,
            oauth_client,
            task_tracker_client,
            task_tracker_service,
            version_control_client,
        })
    }
}
