use crate::application::user::commands::register_via_oauth::command::RegisterUserViaOAuthExecutorCommand;
use crate::application::user::commands::register_via_oauth::error::RegisterUserViaOAuthExecutorError;
use crate::application::user::commands::register_via_oauth::response::RegisterUserViaOAuthExecutorResponse;
use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;
use crate::domain::auth::ports::oauth_client::OAuthClient;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::user::entities::user::User;
use crate::domain::user::entities::user_social::UserSocial;
use crate::domain::user::entities::user_vcs::UserVersionControlService;
use crate::domain::user::ports::version_control_client::VersionControlClient;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_services_repository::UserSocialServicesRepository;
use crate::domain::user::repositories::user_version_control_services::UserVersionControlServicesRepository;
use crate::domain::user::value_objects::version_control_user_id::VersionControlUserId;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::utils::mutex::key_locker::KeyLocker;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::sync::Arc;

pub struct RegisterUserViaOAuthExecutor {
    pub db: Arc<DatabaseConnection>,
    pub user_repo: Arc<dyn UserRepository>,
    pub user_socials_repo: Arc<dyn UserSocialServicesRepository>,
    pub user_version_control_service_repo: Arc<dyn UserVersionControlServicesRepository>,
    pub oauth_client: Arc<dyn OAuthClient>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub notification_service: Arc<dyn NotificationService>,
    pub cache: Arc<dyn CacheService>,
    pub mutex: Arc<KeyLocker<String>>,
}

impl RegisterUserViaOAuthExecutor {
    pub async fn execute(
        &self,
        cmd: RegisterUserViaOAuthExecutorCommand,
    ) -> Result<RegisterUserViaOAuthExecutorResponse, RegisterUserViaOAuthExecutorError> {
        let _guard = self.mutex.lock(cmd.state.clone()).await;

        let txn = self.db.begin().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to begin database transaction");
            RegisterUserViaOAuthExecutorError::UnknownError
        })?;

        let state = self.retrieve_oauth_state(&cmd.state).await?;

        tracing::debug!("{:?}", state);

        if let Ok(..) = self
            .user_socials_repo
            .find_by_social_user_id(&state.social_user_id)
            .await
        {
            return Err(RegisterUserViaOAuthExecutorError::UnknownError);
        }

        let exchange_code_response =
            self.oauth_client
                .exchange_code(&cmd.code)
                .await
                .map_err(|e| {
                    tracing::error!(
                        social_user_id = %state.social_user_id.0,
                        "Failed to exchange code for access token"
                    );
                    RegisterUserViaOAuthExecutorError::UnknownError
                })?;

        let version_control_client_user = self
            .version_control_client
            .get_user(&exchange_code_response.access_token)
            .await
            .map_err(|e| {
                tracing::error!(
                    social_user_id = %state.social_user_id.0,
                    "Failed to exchange code for access token"
                );
                RegisterUserViaOAuthExecutorError::UnknownError
            })?;

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
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    social_user_id = %state.social_user_id.0,
                    "Failed to create User record"
                );
                RegisterUserViaOAuthExecutorError::UnknownError
            })?;

        let new_social_user = UserSocial {
            id: Default::default(),
            user_id: user.id,
            social_type: state.social_type,
            social_user_id: state.social_user_id,
            social_chat_id: state.social_chat_id,
            social_user_login: state.social_user_login,
            social_user_email: state.social_user_email,
            social_user_avatar_url: state.social_user_avatar_url,
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        self.user_socials_repo
            .create(&txn, &new_social_user)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    user_id = %user.id.0,
                    social_user_id = %state.social_user_id.0,
                    "Failed to create UserSocial record{}",
                    format!("{:?}", new_social_user)
                );
                RegisterUserViaOAuthExecutorError::UnknownError
            })?;

        let new_user_version_control_service = UserVersionControlService {
            id: Default::default(),
            user_id: user.id,
            version_control_type: state.version_control_type,
            version_control_user_id: VersionControlUserId(version_control_client_user.id as i32),
            version_control_login: Some(version_control_client_user.login.clone()),
            version_control_email: version_control_client_user.email.clone(),
            version_control_avatar_url: None,
            access_token: Some(exchange_code_response.access_token),
            refresh_token: None,
            token_type: Some(exchange_code_response.token_type),
            scope: Some(exchange_code_response.scope),
            expires_at: None,
            created_at: Default::default(),
            updated_at: Default::default(),
        };

        self.user_version_control_service_repo
            .create(&txn, &new_user_version_control_service)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    user_id = %user.id.0,
                    social_user_id = %state.social_user_id.0,
                    "Failed to create UserVersionControlService record{}",
                    format!("{:?}", new_user_version_control_service)
                );
                RegisterUserViaOAuthExecutorError::UnknownError
            })?;

        self.notification_service
            .send(
                &state.social_type,
                &state.social_chat_id,
                &format!(
                    "Получили данные от гитхаба, распарсили от кого пришло: {} и получили код {}",
                    state.social_user_id.0, cmd.code
                ),
            )
            .await
            .map_err(|_| RegisterUserViaOAuthExecutorError::UnknownError)?;

        txn.commit().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to commit database transaction");
            RegisterUserViaOAuthExecutorError::UnknownError
        })?;

        // TODO
        Ok(RegisterUserViaOAuthExecutorResponse {
            message: String::from("123"),
        })
    }

    async fn retrieve_oauth_state(
        &self,
        key: &str,
    ) -> Result<OpenAuthorizationState, RegisterUserViaOAuthExecutorError> {
        tracing::debug!(state_id = %key, "Retrieving OAuth state from cache");

        let state_json = self
            .cache
            .take(key)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    state_id = %key,
                    "Failed to retrieve state from cache"
                );
                RegisterUserViaOAuthExecutorError::Cache(e.to_string())
            })?
            .ok_or_else(|| {
                tracing::warn!(
                    state_id = %key,
                    "OAuth state not found in cache (expired or invalid)"
                );
                RegisterUserViaOAuthExecutorError::InvalidState
            })?;

        tracing::trace!(
            state_id = %key,
            state_json_length = state_json.len(),
            "OAuth state JSON retrieved from cache"
        );

        serde_json::from_str::<OpenAuthorizationState>(&state_json).map_err(|e| {
            tracing::error!(
                error = %e,
                state_id = %key,
                state_json = %state_json,
                "Failed to deserialize OAuth state JSON"
            );
            RegisterUserViaOAuthExecutorError::Cache(format!("Invalid state JSON: {}", e))
        })
    }
}
