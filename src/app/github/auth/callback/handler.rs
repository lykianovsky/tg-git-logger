use crate::domain::user::use_cases::bind_github::{BindGithubError, BindGithubUseCase};
use crate::domain::user::value_objects::{UserGithubAccount, UserGithubId, UserTelegramId};
use crate::infrastructure::contracts::github::oauth::GithubOAuthState;
use crate::infrastructure::delivery::state::ApplicationState;
use crate::infrastructure::integrations::github::oauth::GithubOAuthError;
use crate::infrastructure::integrations::github::{GithubClientError, GithubUser};
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::CipherError;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
enum GithubAuthorizationError {
    #[error("Invalid state parameter")]
    InvalidState,

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Failed to exchange code for token: {0}")]
    TokenExchange(#[from] GithubOAuthError),

    #[error("Failed to fetch GitHub user: {0}")]
    FetchUser(#[from] GithubClientError),

    #[error("Failed to encrypt token: {0}")]
    Encryption(#[from] CipherError),

    #[error("Failed to bind GitHub account: {0}")]
    BindAccount(#[from] BindGithubError),
}

impl GithubAuthorizationError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidState => StatusCode::BAD_REQUEST,
            Self::BindAccount(BindGithubError::GithubAlreadyBound(_)) => StatusCode::CONFLICT,
            Self::BindAccount(BindGithubError::UserNotFound(_)) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn user_message(&self) -> String {
        match self {
            Self::InvalidState => {
                "Неверная ссылка авторизации. Попробуйте начать сначала.".to_string()
            }
            Self::BindAccount(BindGithubError::GithubAlreadyBound(login)) => {
                format!(
                    "GitHub аккаунт '{}' уже привязан к другому пользователю",
                    login
                )
            }
            Self::BindAccount(BindGithubError::UserNotFound(_)) => {
                "Пользователь не найден. Попробуйте начать сначала.".to_string()
            }
            Self::Cache(_) => {
                "Техническая ошибка. Попробуйте начать авторизацию заново.".to_string()
            }
            _ => "Произошла ошибка при привязке GitHub аккаунта. Попробуйте позже.".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubAuthorizationQuery {
    code: String,
    state: String,
}

pub struct GithubAuthorizationCallbackHandler {}

impl GithubAuthorizationCallbackHandler {
    pub async fn handle(
        State(state): State<Arc<ApplicationState>>,
        Query(query): Query<GithubAuthorizationQuery>,
    ) -> StatusCode {
        tracing::info!(
            state_id = %query.state,
            "GitHub authorization callback received"
        );

        match Self::processing(Arc::clone(&state), query).await {
            Ok(oauth_state) => {
                tracing::info!(
                    telegram_id = oauth_state.telegram_id.0,
                    chat_id = oauth_state.chat_id,
                    "GitHub authorization completed successfully"
                );
                StatusCode::NO_CONTENT
            }
            Err(error) => {
                tracing::error!(
                    error = %error,
                    error_debug = ?error,
                    "GitHub authorization failed"
                );
                error.status_code()
            }
        }
    }

    async fn processing(
        state: Arc<ApplicationState>,
        query: GithubAuthorizationQuery,
    ) -> Result<GithubOAuthState, GithubAuthorizationError> {
        tracing::debug!(
            state_id = %query.state,
            code_length = query.code.len(),
            "Starting GitHub authorization processing"
        );

        // 1. Получаем и валидируем состояние
        let oauth_state = Self::retrieve_oauth_state(&state, &query.state).await?;

        tracing::info!(
            telegram_id = oauth_state.telegram_id.0,
            chat_id = oauth_state.chat_id,
            "OAuth state retrieved successfully"
        );

        // Выполняем основную логику с обработкой ошибок
        let result = Self::process_github_binding(&state, &query.code, &oauth_state).await;

        match &result {
            Ok(github_user) => {
                Self::notify_success(&state, oauth_state.chat_id, &github_user.login).await;

                tracing::info!(
                    telegram_id = oauth_state.telegram_id.0,
                    github_id = github_user.id,
                    github_login = %github_user.login,
                    "GitHub account bound successfully"
                );
            }
            Err(error) => {
                Self::notify_error(&state, oauth_state.chat_id, error).await;

                tracing::warn!(
                    telegram_id = oauth_state.telegram_id.0,
                    chat_id = oauth_state.chat_id,
                    error = %error,
                    "Failed to bind GitHub account, user notified"
                );
            }
        }

        result?;
        Ok(oauth_state)
    }

    async fn process_github_binding(
        state: &ApplicationState,
        code: &str,
        oauth_state: &GithubOAuthState,
    ) -> Result<GithubUser, GithubAuthorizationError> {
        // 2. Обмениваем код на токен
        tracing::debug!("Exchanging authorization code for access token");
        let access_token = Self::exchange_code_for_token(state, code).await?;
        tracing::debug!(token_length = access_token.len(), "Access token received");

        // 3. Получаем данные пользователя GitHub
        tracing::debug!("Fetching GitHub user information");
        let github_user = Self::fetch_github_user(state, &access_token).await?;
        tracing::info!(
            github_id = github_user.id,
            github_login = %github_user.login,
            "GitHub user fetched successfully"
        );

        // 4. Шифруем токен
        tracing::debug!("Encrypting access token");
        let encrypted_token = Self::encrypt_token(state, &access_token)?;
        tracing::debug!(
            encrypted_length = encrypted_token.len(),
            "Access token encrypted"
        );

        // 5. Привязываем аккаунт
        tracing::debug!(
            telegram_id = oauth_state.telegram_id.0,
            github_id = github_user.id,
            "Binding GitHub account to user"
        );

        BindGithubUseCase::new(Arc::clone(&state.repositories.user))
            .execute(
                oauth_state.telegram_id,
                UserGithubAccount {
                    login: github_user.login.clone(),
                    github_id: UserGithubId(github_user.id),
                    access_token: encrypted_token,
                },
            )
            .await
            .map_err(|e| {
                tracing::error!(
                    error = ?e,
                    telegram_id = oauth_state.telegram_id.0,
                    github_login = %github_user.login,
                    "Failed to bind GitHub account"
                );
                GithubAuthorizationError::BindAccount(e)
            })?;

        Ok(github_user)
    }

    async fn retrieve_oauth_state(
        state: &ApplicationState,
        key: &str,
    ) -> Result<GithubOAuthState, GithubAuthorizationError> {
        tracing::debug!(state_id = %key, "Retrieving OAuth state from cache");

        let state_json = state
            .cache
            .take(key)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    state_id = %key,
                    "Failed to retrieve state from cache"
                );
                GithubAuthorizationError::Cache(e.to_string())
            })?
            .ok_or_else(|| {
                tracing::warn!(
                    state_id = %key,
                    "OAuth state not found in cache (expired or invalid)"
                );
                GithubAuthorizationError::InvalidState
            })?;

        tracing::trace!(
            state_id = %key,
            state_json_length = state_json.len(),
            "OAuth state JSON retrieved from cache"
        );

        serde_json::from_str::<GithubOAuthState>(&state_json).map_err(|e| {
            tracing::error!(
                error = %e,
                state_id = %key,
                state_json = %state_json,
                "Failed to deserialize OAuth state JSON"
            );
            GithubAuthorizationError::Cache(format!("Invalid state JSON: {}", e))
        })
    }

    async fn exchange_code_for_token(
        state: &ApplicationState,
        code: &str,
    ) -> Result<String, GithubAuthorizationError> {
        tracing::debug!(
            code_length = code.len(),
            "Requesting access token from GitHub"
        );

        let token_response = state
            .integrations
            .github_oauth_client
            .get_access_token(code)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    error_debug = ?e,
                    "Failed to exchange code for access token"
                );
                GithubAuthorizationError::TokenExchange(e)
            })?;

        tracing::debug!("Access token successfully obtained from GitHub");
        Ok(token_response.access_token)
    }

    async fn fetch_github_user(
        state: &ApplicationState,
        access_token: &str,
    ) -> Result<GithubUser, GithubAuthorizationError> {
        tracing::debug!("Fetching user info from GitHub API");

        state
            .integrations
            .github_client
            .get_user(access_token)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    error_debug = ?e,
                    "Failed to fetch GitHub user information"
                );
                GithubAuthorizationError::FetchUser(e)
            })
    }

    fn encrypt_token(
        state: &ApplicationState,
        token: &str,
    ) -> Result<String, GithubAuthorizationError> {
        tracing::debug!(token_length = token.len(), "Encrypting access token");

        state.cipher.encrypt(token).map_err(|e| {
            tracing::error!(
                error = %e,
                error_debug = ?e,
                "Failed to encrypt access token"
            );
            GithubAuthorizationError::Encryption(e)
        })
    }

    async fn notify_success(state: &ApplicationState, chat_id: i64, github_login: &str) {
        tracing::debug!(
            chat_id = chat_id,
            github_login = %github_login,
            "Sending success notification to user"
        );

        let message = MessageBuilder::new().line(&format!(
            "✅ GitHub пользователь {} успешно привязан",
            github_login
        ));

        if let Err(e) = state
            .services
            .notifier
            .send_to_chat(chat_id, &message)
            .await
        {
            tracing::warn!(
                error = ?e,
                chat_id = chat_id,
                "Failed to send success notification to user"
            );
        } else {
            tracing::debug!(chat_id = chat_id, "Success notification sent");
        }
    }

    async fn notify_error(
        state: &ApplicationState,
        chat_id: i64,
        error: &GithubAuthorizationError,
    ) {
        tracing::debug!(
            chat_id = chat_id,
            error = %error,
            "Sending error notification to user"
        );

        let message = MessageBuilder::new()
            .line("❌ Ошибка при привязке GitHub")
            .line(&error.user_message());

        if let Err(e) = state
            .services
            .notifier
            .send_to_chat(chat_id, &message)
            .await
        {
            tracing::error!(
                error = ?e,
                chat_id = chat_id,
                original_error = %error,
                "Failed to send error notification to user"
            );
        } else {
            tracing::debug!(chat_id = chat_id, "Error notification sent");
        }
    }
}
