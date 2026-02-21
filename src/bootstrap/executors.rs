use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::application::webhook::commands::notify_received_event::executor::NotifyReceivedWebhookEventExecutor;
use crate::config::application::ApplicationConfig;
use crate::domain::shared::events::publisher::EventPublisher;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::infrastructure::drivers::cache::redis::RedisCache;
use crate::infrastructure::integrations::oauth::github::GithubOAuthClient;
use crate::infrastructure::integrations::version_control::github::GithubVersionControlClient;
use crate::infrastructure::repositories::mysql::user::MySQLUserRepository;
use crate::infrastructure::repositories::mysql::user_has_roles::MySQLUserHasRolesRepository;
use crate::infrastructure::repositories::mysql::user_social_services::MySQLUserSocialServicesRepository;
use crate::infrastructure::repositories::mysql::user_version_control_services::MySQLUserVersionControlServicesRepository;
use crate::infrastructure::services::notification::CompositionNotificationService;
use crate::utils::mutex::key_locker::KeyLocker;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ApplicationBoostrapExecutorsQueries {}

pub struct ApplicationBoostrapExecutorsCommands {
    pub register_user_via_oauth: Arc<RegisterUserViaOAuthExecutor>,
    pub create_oauth_link: Arc<CreateOAuthLinkExecutor>,
    pub dispatch_webhook_event: Arc<DispatchWebhookEventExecutor>,
    pub notify_received_webhook_event: Arc<NotifyReceivedWebhookEventExecutor>,
}

pub struct ApplicationBoostrapExecutors {
    pub queries: ApplicationBoostrapExecutorsQueries,
    pub commands: ApplicationBoostrapExecutorsCommands,
}

impl ApplicationBoostrapExecutors {
    pub fn new(
        config: Arc<ApplicationConfig>,
        mysql_pool: Arc<DatabaseConnection>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        let cache: Arc<dyn CacheService> = Arc::new(RedisCache::new(config.redis.url.clone()));

        let role_repo = Arc::new(MySQLUserRepository::new(mysql_pool.clone()));

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

        let oauth_client = Arc::new(GithubOAuthClient::new());
        let version_control_client = Arc::new(GithubVersionControlClient::new());

        let queries = ApplicationBoostrapExecutorsQueries {};

        let commands = ApplicationBoostrapExecutorsCommands {
            create_oauth_link: Arc::new(CreateOAuthLinkExecutor::new(
                event_publisher.clone(),
                user_repo.clone(),
                user_socials_repo.clone(),
                cache.clone(),
            )),
            register_user_via_oauth: Arc::new(RegisterUserViaOAuthExecutor {
                db: mysql_pool.clone(),
                user_repo: user_repo.clone(),
                user_socials_repo: user_socials_repo.clone(),
                user_version_control_service_repo: user_version_controls_repo.clone(),
                oauth_client: oauth_client.clone(),
                version_control_client: version_control_client.clone(),
                notification_service: notification_service.clone(),
                cache: cache.clone(),
                mutex: Arc::new(KeyLocker::new()),
            }),
            dispatch_webhook_event: Arc::new(DispatchWebhookEventExecutor {
                publisher: event_publisher.clone(),
            }),
            notify_received_webhook_event: Arc::new(NotifyReceivedWebhookEventExecutor::new(
                notification_service.clone(),
            )),
        };

        Self { queries, commands }
    }
}
