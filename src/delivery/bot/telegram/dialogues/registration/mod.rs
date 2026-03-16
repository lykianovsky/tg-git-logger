use crate::application::auth::commands::create_oauth_link::command::{
    CreateOAuthLinkExecutorCommand, CreateOAuthLinkExecutorCommandSocial,
    CreateOAuthLinkExecutorCommandVersionControl,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::delivery::bot::telegram::keyboards::actions::choose_role::TelegramBotChooseRoleAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use crate::utils::builder::message::MessageBuilder;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use teloxide::{dptree, Bot};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueRegistrationState {
    #[default]
    ChooseRole,
}

pub struct TelegramBotDialogueRegistrationDispatcher {}

impl TelegramBotDialogueRegistrationDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query().branch(
            case![TelegramBotDialogueRegistrationState::ChooseRole]
                .endpoint(TelegramBotDialogueRegistrationDispatcher::choose_role),
        );

        dptree::entry().branch(queries)
    }

    async fn choose_role(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id).await?;

        let callback_data = query.data.as_deref().unwrap_or("");

        let selected_role = match TelegramBotChooseRoleAction::from_callback_data(callback_data) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{e}");
                return Ok(());
            }
        };

        let user = query.from;
        let msg = query.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(
            chat_id,
            message_id,
            format!("Выбранная роль: {}", selected_role.to_callback_data()),
        )
        .await?;

        let cmd = CreateOAuthLinkExecutorCommand {
            social: CreateOAuthLinkExecutorCommandSocial {
                r#type: SocialType::Telegram,
                chat_id: SocialChatId(chat_id.0),
                user_id: SocialUserId(user.id.0 as i32),
                user_login: user.username.clone(),
                user_email: None,
                user_avatar_url: None,
            },
            version_control: CreateOAuthLinkExecutorCommandVersionControl {
                r#type: VersionControlType::Github,
                base: config.github.base.to_string(),
                path: config.github.oauth_pathname.clone(),
                client_id: config.github.oauth_client_id.clone(),
                scope: config.github.oauth_client_scope.to_string(),
            },
            role: selected_role.into(),
        };

        match executors.commands.create_oauth_link.execute(&cmd).await {
            Ok(response) => {
                let message = MessageBuilder::new()
                    .line(t!("telegram_bot.commands.register.title").as_ref())
                    .empty_line()
                    .link(
                        t!("telegram_bot.commands.register.body").as_ref(),
                        response.url.as_str(),
                    )
                    .empty_line()
                    .line(t!("telegram_bot.commands.register.expiration_time").as_ref());

                bot.send_message(chat_id, message)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            Err(error) => {
                bot.send_message(chat_id, error.to_string()).await?;
            }
        }

        dialogue.exit().await.ok();

        Ok(())
    }
}
