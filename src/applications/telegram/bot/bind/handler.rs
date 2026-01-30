use crate::applications::telegram::bot::commands::Command;
use crate::applications::telegram::bot::context::TelegramBotCommandContext;
use crate::domain::user::entities::User;
use crate::domain::user::use_cases::bind_github::BindGithubUseCase;
use crate::domain::user::use_cases::create::CreateUserUseCase;
use crate::domain::user::value_objects::{UserGithubAccount, UserGithubId, UserId, UserRole, UserTelegramAccount, UserTelegramId};
use regex::Regex;
use std::sync::Arc;
use teloxide::prelude::{Message, Requester};
use teloxide::RequestError;

pub struct TelegramBotBindCommandHandler {
    context: TelegramBotCommandContext
}

impl TelegramBotBindCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext
    ) -> Self {
        Self {
            context
        }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        let text = match self.context.msg.text() {
            Some(text) => {
                text
            }
            None => {
                return self.context.bot.send_message(
                    self.context.msg.chat.id,
                    "Мы не смогли обработать сообщение, попробуйте позже",
                ).await
            }
        };
        let parts = text.split_whitespace().collect::<Vec<&str>>();

        if parts.len() < 2 {
            return self.context.bot.send_message(
                self.context.msg.chat.id,
                format!("❌ Укажи GitHub логин в формате: /{} your_login", Command::Bind.as_str()),
            ).await
        }

        let github_login = parts[1];

        let github_login_pattern = Regex::new(r"^[a-zA-Z0-9-]{1,39}$").unwrap();

        tracing::debug!("{}", github_login);

        if !github_login_pattern.is_match(github_login) {
            return self.context.bot
                .send_message(
                    self.context.msg.chat.id,
                    "❌ Неверный формат GitHub логина. Допустимы только буквы, цифры и '-' (максимум 39 символов).",
                )
                .await;
        }

        return match BindGithubUseCase::new(
            Arc::clone(&self.context.application_state.repositories.user)
        ).execute(UserTelegramId(self.context.user.id.0 as i32), github_login.to_string()).await {
            Ok(..) => {
                tracing::info!("GitHub account '{}' successfully bound to user ID {}", github_login, self.context.user.id.0);
                self.context.bot.send_message(
                    self.context.msg.chat.id,
                    format!("✅ GitHub аккаунт '{}' успешно привязан к вашему профилю.", github_login),
                ).await
            }
            Err(e) => {
                tracing::error!("Error creating BindGithubUseCase: {:?}", e);
                self.context.bot.send_message(
                    self.context.msg.chat.id,
                    "❌ Произошла ошибка при создании обработчика привязки GitHub аккаунта.",
                ).await
            }
        }
    }
}