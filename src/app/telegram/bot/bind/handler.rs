use crate::app::telegram::bot::context::TelegramBotCommandContext;
use crate::config::environment::ENV;
use crate::domain::user::value_objects::UserTelegramId;
use crate::infrastructure::contracts::github::oauth::GithubOAuthState;
use crate::utils::builder::message::MessageBuilder;
use redis::RedisError;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ParseMode;
use teloxide::RequestError;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum TelegramBotBindError {
    #[error("Failed to access cache: {0}")]
    Cache(#[from] RedisError),

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Failed to serialize state: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Failed to send Telegram message: {0}")]
    TelegramRequest(#[from] RequestError),

    #[error("Failed to check existing binding: {0}")]
    DatabaseQuery(String),
}

impl TelegramBotBindError {
    fn user_message(&self) -> &str {
        match self {
            Self::Cache(_) | Self::Serialization(_) => {
                "Техническая ошибка. Попробуйте позже или обратитесь к администратору."
            }
            Self::UrlParse(_) => {
                "Не удалось создать ссылку для авторизации. Обратитесь к администратору."
            }
            Self::DatabaseQuery(_) => "Ошибка при проверке привязки. Попробуйте еще раз.",
            Self::TelegramRequest(_) => "Не удалось отправить сообщение. Попробуйте еще раз.",
        }
    }
}

const OAUTH_STATE_TTL_SECONDS: u64 = 600;

pub struct TelegramBotBindCommandHandler {
    context: TelegramBotCommandContext,
}

impl TelegramBotBindCommandHandler {
    pub fn new(context: TelegramBotCommandContext) -> Self {
        Self { context }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        let telegram_id = UserTelegramId(self.context.user.id.0 as i32);
        let chat_id = self.context.msg.chat.id;

        tracing::info!(
            telegram_id = telegram_id.0,
            chat_id = chat_id.0,
            username = ?self.context.user.username,
            "Received /bind command"
        );

        match self.process_bind_command(telegram_id, chat_id).await {
            Ok(message) => {
                tracing::info!(
                    telegram_id = telegram_id.0,
                    chat_id = chat_id.0,
                    "Bind command processed successfully"
                );
                Ok(message)
            }
            Err(error) => {
                tracing::error!(
                    error = %error,
                    error_debug = ?error,
                    telegram_id = telegram_id.0,
                    chat_id = chat_id.0,
                    "Failed to process bind command"
                );

                self.send_error_message(chat_id, &error).await
            }
        }
    }

    async fn process_bind_command(
        &self,
        telegram_id: UserTelegramId,
        chat_id: teloxide::types::ChatId,
    ) -> Result<Message, TelegramBotBindError> {
        // 1. Проверяем существующую привязку
        tracing::debug!(
            telegram_id = telegram_id.0,
            "Checking for existing GitHub binding"
        );

        if let Some(existing_message) = self.check_existing_binding(&telegram_id).await? {
            tracing::info!(
                telegram_id = telegram_id.0,
                "User already has GitHub account bound"
            );
            return Ok(existing_message);
        }

        // 2. Создаем OAuth состояние
        let state = GithubOAuthState {
            telegram_id,
            chat_id: chat_id.0,
        };

        tracing::debug!(
            telegram_id = telegram_id.0,
            chat_id = chat_id.0,
            "Creating OAuth URL"
        );

        // 3. Создаем OAuth URL
        let url = self.create_oauth_url(&state).await?;

        tracing::info!(
            telegram_id = telegram_id.0,
            state_ttl = OAUTH_STATE_TTL_SECONDS,
            "OAuth URL created successfully"
        );

        // 4. Отправляем сообщение с URL
        self.send_oauth_link_message(chat_id, &url).await
    }

    async fn create_oauth_url(
        &self,
        state: &GithubOAuthState,
    ) -> Result<Url, TelegramBotBindError> {
        tracing::debug!(
            telegram_id = state.telegram_id.0,
            "Constructing GitHub OAuth URL"
        );

        // Создаем базовый URL
        let github_base = ENV.get("GITHUB_BASE");
        let mut url =
            Url::parse(&format!("{}/login/oauth/authorize", github_base)).map_err(|e| {
                tracing::error!(
                    error = %e,
                    github_base = %github_base,
                    "Failed to parse GitHub base URL"
                );
                e
            })?;

        // Генерируем ключ состояния
        let state_key = self
            .context
            .application_state
            .cache
            .make_key_by_payload(state);

        tracing::trace!(
            state_key = %state_key,
            telegram_id = state.telegram_id.0,
            "Generated OAuth state key"
        );

        // Сериализуем состояние
        let state_stringify = serde_json::to_string(state).map_err(|e| {
            tracing::error!(
                error = %e,
                telegram_id = state.telegram_id.0,
                "Failed to serialize OAuth state"
            );
            e
        })?;

        // Добавляем параметры запроса
        let client_id = ENV.get("GITHUB_OAUTH_CLIENT_ID");
        let scope = ENV.get("GITHUB_OAUTH_SCOPE");

        url.query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("scope", &scope)
            .append_pair("state", &state_key);

        tracing::debug!(
            state_key = %state_key,
            client_id = %client_id,
            scope = %scope,
            "OAuth URL parameters set"
        );

        // Сохраняем состояние в кэш
        self.context
            .application_state
            .cache
            .set(&state_key, &state_stringify, OAUTH_STATE_TTL_SECONDS)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    state_key = %state_key,
                    ttl = OAUTH_STATE_TTL_SECONDS,
                    "Failed to save OAuth state to cache"
                );
                e
            })?;

        tracing::debug!(
            state_key = %state_key,
            ttl = OAUTH_STATE_TTL_SECONDS,
            "OAuth state saved to cache"
        );

        Ok(url)
    }

    async fn check_existing_binding(
        &self,
        telegram_id: &UserTelegramId,
    ) -> Result<Option<Message>, TelegramBotBindError> {
        tracing::debug!(
            telegram_id = telegram_id.0,
            "Querying database for existing binding"
        );

        let user = self
            .context
            .application_state
            .repositories
            .user
            .find_by_tg_id(*telegram_id)
            .await
            .map_err(|e| {
                tracing::warn!(
                    error = ?e,
                    telegram_id = telegram_id.0,
                    "Failed to query user from database"
                );
                TelegramBotBindError::DatabaseQuery(format!("{:?}", e))
            })?;

        if let Some(github) = user.github {
            tracing::info!(
                telegram_id = telegram_id.0,
                github_login = %github.login,
                github_id = github.github_id.0,
                "User already has GitHub account bound"
            );

            let message = self
                .context
                .bot
                .send_message(
                    self.context.msg.chat.id,
                    format!(
                        "✅ У вас уже привязан GitHub аккаунт\n\nUsername: {}",
                        github.login
                    ),
                )
                .await
                .map_err(|e| {
                    tracing::error!(
                        error = ?e,
                        telegram_id = telegram_id.0,
                        "Failed to send existing binding message"
                    );
                    e
                })?;

            return Ok(Some(message));
        }

        tracing::debug!(
            telegram_id = telegram_id.0,
            "No existing GitHub binding found"
        );

        Ok(None)
    }

    async fn send_oauth_link_message(
        &self,
        chat_id: teloxide::types::ChatId,
        url: &Url,
    ) -> Result<Message, TelegramBotBindError> {
        tracing::debug!(
            chat_id = chat_id.0,
            url_length = url.as_str().len(),
            "Sending OAuth link message to user"
        );

        let message = MessageBuilder::new()
            .line("🔗 Для привязки GitHub аккаунта:")
            .line("")
            .link("👉 Авторизоваться через GitHub", url.as_str())
            .line("")
            .line("⏱ Ссылка действительна 10 минут");

        let sent_message = self
            .context
            .bot
            .send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .await
            .map_err(|e| {
                tracing::error!(
                    error = ?e,
                    chat_id = chat_id.0,
                    "Failed to send OAuth link message"
                );
                e
            })?;

        tracing::debug!(
            chat_id = chat_id.0,
            message_id = sent_message.id.0,
            "OAuth link message sent successfully"
        );

        Ok(sent_message)
    }

    async fn send_error_message(
        &self,
        chat_id: teloxide::types::ChatId,
        error: &TelegramBotBindError,
    ) -> Result<Message, RequestError> {
        tracing::debug!(
            chat_id = chat_id.0,
            error = %error,
            "Sending error message to user"
        );

        let message = MessageBuilder::new()
            .line("❌ Ошибка")
            .line("")
            .line(error.user_message());

        let sent_message = self
            .context
            .bot
            .send_message(chat_id, message)
            .parse_mode(ParseMode::Html)
            .await?;

        tracing::debug!(
            chat_id = chat_id.0,
            message_id = sent_message.id.0,
            "Error message sent to user"
        );

        Ok(sent_message)
    }
}
