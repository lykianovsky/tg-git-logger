use crate::application::user::commands::register_via_oauth::command::RegisterUserViaOAuthExecutorCommand;
use crate::application::user::commands::register_via_oauth::error::RegisterUserViaOAuthExecutorError;
use crate::application::user::commands::register_via_oauth::response::RegisterUserViaOAuthExecutorResponse;
use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;
use crate::domain::auth::ports::oauth_client::OAuthClient;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::entities::user::User;
use crate::domain::user::entities::user_social_account::UserSocialAccount;
use crate::domain::user::entities::user_vc_account::UserVersionControlAccount;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::version_control_user_id::VersionControlUserId;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::utils::builder::message::MessageBuilder;
use crate::utils::mutex::key_locker::KeyLocker;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct RegisterUserViaOAuthExecutor {
    pub db: Arc<DatabaseConnection>,
    pub user_has_role: Arc<dyn UserHasRolesRepository>,
    pub user_repo: Arc<dyn UserRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_version_control_service_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub oauth_client: Arc<dyn OAuthClient>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub reversible_cipher: Arc<ReversibleCipher>,
    pub cache: Arc<dyn CacheService>,
    pub mutex: Arc<KeyLocker<String>>,
}

impl CommandExecutor for RegisterUserViaOAuthExecutor {
    type Command = RegisterUserViaOAuthExecutorCommand;
    type Response = RegisterUserViaOAuthExecutorResponse;
    type Error = RegisterUserViaOAuthExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let _guard = self.mutex.lock(cmd.code.clone()).await;

        let txn = self.db.begin().await?;

        tracing::debug!("{:?}", cmd.state);

        if let Ok(..) = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.state.social_user_id)
            .await
        {
            return Err(
                RegisterUserViaOAuthExecutorError::UserBySocialUserIdAlreadyExists(
                    cmd.state.social_user_id.0,
                ),
            );
        }

        let exchange_code_response = self.oauth_client.exchange_code(&cmd.code).await?;

        let version_control_client_user = self
            .version_control_client
            .get_user(&exchange_code_response.access_token)
            .await?;

        let user = self
            .user_repo
            .create(
                &txn,
                &User {
                    id: Default::default(),
                    is_active: true,
                    create_at: Default::default(),
                    update_at: Default::default(),
                },
            )
            .await?;

        self.user_has_role
            .assign(&txn, user.id, cmd.state.role.clone())
            .await?;

        let new_social_user = UserSocialAccount {
            id: Default::default(),
            user_id: user.id,
            social_type: cmd.state.social_type.clone(),
            social_user_id: cmd.state.social_user_id,
            social_chat_id: cmd.state.social_chat_id,
            social_user_login: cmd.state.social_user_login.clone(),
            social_user_email: cmd.state.social_user_email.clone(),
            social_user_avatar_url: cmd.state.social_user_avatar_url.clone(),
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        self.user_socials_repo
            .create(&txn, &new_social_user)
            .await?;

        let encrypted_access_token = self
            .reversible_cipher
            .encrypt(&exchange_code_response.access_token)?;

        let new_user_version_control_service = UserVersionControlAccount {
            id: Default::default(),
            user_id: user.id,
            version_control_type: cmd.state.version_control_type.clone(),
            version_control_user_id: VersionControlUserId(version_control_client_user.id as i32),
            version_control_login: version_control_client_user.login.clone(),
            version_control_email: version_control_client_user.email.clone(),
            version_control_avatar_url: None,
            access_token: encrypted_access_token,
            refresh_token: None,
            token_type: Some(exchange_code_response.token_type),
            scope: Some(exchange_code_response.scope),
            expires_at: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        self.user_version_control_service_repo
            .create(&txn, &new_user_version_control_service)
            .await?;

        txn.commit().await?;

        Ok(RegisterUserViaOAuthExecutorResponse {
            user,
            user_social_account: new_social_user,
            user_version_control_account: new_user_version_control_service,
        })
    }
}
