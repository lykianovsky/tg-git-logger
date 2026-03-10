use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::application::notification::commands::send_social_notify::executor::SendSocialNotifyExecutor;
use crate::application::task::commands::move_task_to_test::executor::MoveTaskToTestExecutor;
use crate::application::user::commands::register_via_oauth::executor::RegisterUserViaOAuthExecutor;
use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::application::webhook::commands::dispatch_event::executor::DispatchWebhookEventExecutor;
use crate::config::application::ApplicationConfig;
use crate::domain::auth::ports::oauth_client::OAuthClient;
use crate::domain::task::ports::task_tracker_client::TaskTrackerClient;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::infrastructure::drivers::cache::redis::RedisCache;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::infrastructure::integrations::oauth::github::GithubOAuthClient;
use crate::infrastructure::integrations::task_tracker::kaiten::{
    KaitenClient, KaitenClientBase, KaitenClientToken,
};
use crate::infrastructure::integrations::version_control::github::client::GithubVersionControlClient;
use crate::infrastructure::repositories::mysql::user::MySQLUserRepository;
use crate::infrastructure::repositories::mysql::user_has_roles::MySQLUserHasRolesRepository;
use crate::infrastructure::repositories::mysql::user_social_accounts::MySQLUserSocialServicesRepository;
use crate::infrastructure::repositories::mysql::user_vc_accounts::MySQLUserVersionControlServicesRepository;
use crate::infrastructure::services::notification::CompositionNotificationService;
use crate::infrastructure::services::task_tracker::kaiten::KaitenTaskTrackerService;
use crate::utils::mutex::key_locker::KeyLocker;
use crate::utils::security::crypto::reversible::ReversibleCipher;
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
        publisher: Arc<dyn MessageBrokerPublisher>,
    ) -> Self {
        let reversible_cipher = Arc::new(ReversibleCipher::new(
            &config.secret.reversible_cipher_secret,
        ));

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

        let queries = ApplicationBoostrapExecutorsQueries {
            build_report_by_range: Arc::new(BuildVersionControlDateRangeReportExecutor {
                reversible_cipher: reversible_cipher.clone(),
                user_socials_repo: user_socials_repo.clone(),
                user_version_control_service_repo: user_version_controls_repo.clone(),
                version_control_client: version_control_client.clone(),
            }),
        };

        let commands = ApplicationBoostrapExecutorsCommands {
            create_oauth_link: Arc::new(CreateOAuthLinkExecutor::new(
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
                reversible_cipher: reversible_cipher.clone(),
                notification_service: notification_service.clone(),
                cache: cache.clone(),
                mutex: Arc::new(KeyLocker::new()),
            }),
            dispatch_webhook_event: Arc::new(DispatchWebhookEventExecutor {
                publisher: publisher.clone(),
            }),
            send_social_notify: Arc::new(SendSocialNotifyExecutor::new(
                notification_service.clone(),
            )),
            move_task_to_test: Arc::new(MoveTaskToTestExecutor::new(
                task_tracker_client.clone(),
                task_tracker_service.clone(),
                config.task_tracker.test_column_id,
            )),
        };

        Self { queries, commands }
    }
}
