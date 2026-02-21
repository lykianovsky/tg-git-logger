use crate::application::auth::commands::create_oauth_link::command::{
    CreateOAuthLinkExecutorCommand, CreateOAuthLinkExecutorCommandSocial,
    CreateOAuthLinkExecutorCommandVersionControl,
};
use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;
use teloxide::RequestError;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{Message, ParseMode};

pub struct TelegramBotRegisterCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<CreateOAuthLinkExecutor>,
}

impl TelegramBotRegisterCommandHandler {
    pub fn new(context: TelegramBotCommandContext, executor: Arc<CreateOAuthLinkExecutor>) -> Self {
        Self { context, executor }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        let cmd = CreateOAuthLinkExecutorCommand {
            social: CreateOAuthLinkExecutorCommandSocial {
                r#type: SocialType::Telegram,
                chat_id: SocialChatId(self.context.msg.chat.id.0),
                user_id: SocialUserId(self.context.user.id.0 as i32),
                user_login: self.context.user.username.clone(),
                user_email: None,
                user_avatar_url: None,
            },
            version_control: CreateOAuthLinkExecutorCommandVersionControl {
                r#type: VersionControlType::Github,
                // TODO
                base: String::from("https://github.com"),
                path: String::from("/login/oauth/authorize"),
                client_id: String::from("Ov23liw3t4P8ctoy6vMv"),
                scope: String::from("user,repo"),
            },
        };

        match self.executor.execute(cmd).await {
            Ok(response) => {
                let message = MessageBuilder::new()
                    .line("🔗 Для привязки GitHub аккаунта:")
                    .empty_line()
                    .link("👉 Авторизоваться через GitHub", response.url.as_str())
                    .empty_line()
                    .line("⏱ Ссылка действительна 10 минут");

                return self
                    .context
                    .bot
                    .send_message(self.context.msg.chat.id, message)
                    .parse_mode(ParseMode::Html)
                    .await;
            }
            Err(error) => {
                self.context
                    .bot
                    .send_message(self.context.msg.chat.id, error.to_string())
                    .await
            }
        }
    }
}
