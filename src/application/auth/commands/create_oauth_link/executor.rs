use crate::application::auth::commands::create_oauth_link::command::CreateOAuthLinkExecutorCommand;
use crate::application::auth::commands::create_oauth_link::error::CreateOAuthLinkExecutorError;
use crate::application::auth::commands::create_oauth_link::response::CreateOAuthLinkExecutorResponse;
use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;
use crate::domain::shared::events::publisher::EventPublisher;
use crate::domain::user::events::user_register_notify::UserRegisterNotifyEvent;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_services_repository::UserSocialServicesRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::utils::security::crypto::key_by_payload::create_key_by_payload;
use std::sync::Arc;
use url::Url;

const OAUTH_STATE_TTL_SECONDS: u64 = 600;

pub struct CreateOAuthLinkExecutor {
    publisher: Arc<dyn EventPublisher>,
    user_repo: Arc<dyn UserRepository>,
    user_socials_repo: Arc<dyn UserSocialServicesRepository>,
    cache: Arc<dyn CacheService>,
}

impl CreateOAuthLinkExecutor {
    pub fn new(
        publisher: Arc<dyn EventPublisher>,
        user_repo: Arc<dyn UserRepository>,
        user_socials_repo: Arc<dyn UserSocialServicesRepository>,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self {
            publisher,
            user_repo,
            user_socials_repo,
            cache,
        }
    }

    pub async fn execute(
        &self,
        cmd: CreateOAuthLinkExecutorCommand,
    ) -> Result<CreateOAuthLinkExecutorResponse, CreateOAuthLinkExecutorError> {
        self.publisher
            .publish(&UserRegisterNotifyEvent {
                social_chat_id: SocialChatId(0),
                social_type: SocialType::Telegram,
            })
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::UnknownError)?;

        if let Ok(..) = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social.user_id)
            .await
        {
            return Err(CreateOAuthLinkExecutorError::ExistRegisteredSocialAccountError);
        }

        let link = self.create_oauth_link(&cmd).await?;

        Ok(CreateOAuthLinkExecutorResponse {
            url: link.to_string(),
        })
    }

    pub async fn create_oauth_link(
        &self,
        cmd: &CreateOAuthLinkExecutorCommand,
    ) -> Result<Url, CreateOAuthLinkExecutorError> {
        let mut url = Url::parse(&format!(
            "{}{}",
            cmd.version_control.base, cmd.version_control.path
        ))
        .map_err(|e| {
            tracing::error!(
                error = %e,
                github_base = %cmd.version_control.base,
                "Failed to parse GitHub base URL"
            );
            e
        })?;

        let state = OpenAuthorizationState {
            version_control_type: cmd.version_control.r#type.clone(),
            social_user_id: cmd.social.user_id,
            social_type: cmd.social.r#type,
            social_chat_id: cmd.social.chat_id,
            social_user_email: cmd.social.user_email.clone(),
            social_user_login: cmd.social.user_login.clone(),
            social_user_avatar_url: cmd.social.user_avatar_url.clone(),
        };

        let secret = format!(
            "{}{}{}",
            state.social_user_id.0, state.social_type, state.social_chat_id.0
        );

        let state_key = create_key_by_payload(&secret, &state);

        if let Some(..) = self
            .cache
            .get(&state_key.0)
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::Cache(e.to_string()))?
        {
            return Err(CreateOAuthLinkExecutorError::Cache(String::from(
                "Cache has exist, try later",
            )));
        }

        url.query_pairs_mut()
            .append_pair("client_id", &cmd.version_control.client_id)
            .append_pair("scope", &cmd.version_control.scope)
            .append_pair("state", &state_key.0);

        let str = serde_json::to_string(&state)?;

        self.cache
            .set(&state_key.0, &str, OAUTH_STATE_TTL_SECONDS)
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::Cache(e.to_string()))?;

        tracing::debug!(
            state_key = %state_key.0,
            ttl = OAUTH_STATE_TTL_SECONDS,
            "OAuth state saved to cache"
        );

        Ok(url)
    }
}
