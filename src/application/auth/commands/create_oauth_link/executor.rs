use crate::application::auth::commands::create_oauth_link::command::CreateOAuthLinkExecutorCommand;
use crate::application::auth::commands::create_oauth_link::error::CreateOAuthLinkExecutorError;
use crate::application::auth::commands::create_oauth_link::response::CreateOAuthLinkExecutorResponse;
use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::infrastructure::drivers::cache::contract::CacheService;
use rand::RngCore;
use rand::rngs::OsRng;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

const OAUTH_STATE_TTL_SECONDS: u64 = Duration::from_mins(10).as_secs();

pub struct CreateOAuthLinkExecutor {
    user_repo: Arc<dyn UserRepository>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    cache: Arc<dyn CacheService>,
}

impl CreateOAuthLinkExecutor {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self {
            user_repo,
            user_socials_repo,
            cache,
        }
    }

    async fn create_oauth_link(
        &self,
        cmd: &CreateOAuthLinkExecutorCommand,
    ) -> Result<Url, CreateOAuthLinkExecutorError> {
        let mut url = Url::parse(&format!(
            "{}{}",
            cmd.version_control.base, cmd.version_control.path
        ))?;

        let state = OpenAuthorizationState {
            version_control_type: cmd.version_control.r#type.clone(),
            social_user_id: cmd.social.user_id,
            social_type: cmd.social.r#type,
            social_chat_id: cmd.social.chat_id,
            social_message_id: cmd.social.message_id,
            social_user_email: cmd.social.user_email.clone(),
            social_user_login: cmd.social.user_login.clone(),
            social_user_avatar_url: cmd.social.user_avatar_url.clone(),
            role: cmd.role.clone(),
        };

        let pending_key = format!("oauth_pending:social:{}", state.social_user_id.0);
        if self
            .cache
            .get(&pending_key)
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::Cache(e.to_string()))?
            .is_some()
        {
            return Err(CreateOAuthLinkExecutorError::CacheHasExist(pending_key));
        }

        let mut state_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut state_bytes);
        let state_key = hex::encode(state_bytes);

        url.query_pairs_mut()
            .append_pair("client_id", &cmd.version_control.client_id)
            .append_pair("scope", &cmd.version_control.scope)
            .append_pair("state", &state_key);

        let str = serde_json::to_string(&state)?;

        self.cache
            .set(&state_key, &str, OAUTH_STATE_TTL_SECONDS)
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::Cache(e.to_string()))?;

        self.cache
            .set(&pending_key, &state_key, OAUTH_STATE_TTL_SECONDS)
            .await
            .map_err(|e| CreateOAuthLinkExecutorError::Cache(e.to_string()))?;

        tracing::debug!(
            ttl = OAUTH_STATE_TTL_SECONDS,
            "OAuth state saved to cache"
        );

        Ok(url)
    }
}

impl CommandExecutor for CreateOAuthLinkExecutor {
    type Command = CreateOAuthLinkExecutorCommand;
    type Response = CreateOAuthLinkExecutorResponse;
    type Error = CreateOAuthLinkExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        if let Ok(..) = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social.user_id)
            .await
        {
            return Err(
                CreateOAuthLinkExecutorError::ExistRegisteredSocialAccountError(
                    cmd.social.user_id.0.to_string(),
                ),
            );
        }

        let link = self.create_oauth_link(cmd).await?;

        Ok(CreateOAuthLinkExecutorResponse {
            url: link.to_string(),
        })
    }
}
