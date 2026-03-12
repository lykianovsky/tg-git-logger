use crate::application::auth::commands::create_oauth_link::command::{
    CreateOAuthLinkExecutorCommand, CreateOAuthLinkExecutorCommandSocial,
    CreateOAuthLinkExecutorCommandVersionControl,
};
use crate::application::auth::commands::create_oauth_link::executor::CreateOAuthLinkExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::registration::TelegramBotDialogueRegistrationState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::choose_role::TelegramBotChooseRoleAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::version_control_type::VersionControlType;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub struct TelegramBotRegisterCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<CreateOAuthLinkExecutor>,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotRegisterCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executor: Arc<CreateOAuthLinkExecutor>,
        dialogue: Arc<TelegramBotDialogueType>,
    ) -> Self {
        Self {
            context,
            executor,
            dialogue,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _cmd = CreateOAuthLinkExecutorCommand {
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
                base: self.context.config.github.base.to_string(),
                path: self.context.config.github.oauth_pathname.clone(),
                client_id: self.context.config.github.oauth_client_id.clone(),
                scope: self.context.config.github.oauth_client_scope.to_string(),
            },
        };

        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotChooseRoleAction>(vec![
                TelegramBotChooseRoleAction::Developer,
                TelegramBotChooseRoleAction::QualityAssurance,
            ])
            .build();

        self.dialogue
            .update(TelegramBotDialogueState::Registration(
                TelegramBotDialogueRegistrationState::ChooseRole,
            ))
            .await?;

        self.context
            .bot
            .send_message(self.context.msg.chat.id, "💼 Ваша должность?")
            .reply_markup(keyboard)
            .await?;

        // match self.executor.execute(&cmd).await {
        //     Ok(response) => {
        //         let message = MessageBuilder::new()
        //             .line(t!("telegram_bot.commands.register.title").as_ref())
        //             .empty_line()
        //             .link(
        //                 t!("telegram_bot.commands.register.body").as_ref(),
        //                 response.url.as_str(),
        //             )
        //             .empty_line()
        //             .line(t!("telegram_bot.commands.register.expiration_time").as_ref());
        //
        //         self.context
        //             .bot
        //             .send_message(self.context.msg.chat.id, message)
        //             .parse_mode(ParseMode::Html)
        //             .await?;
        //     }
        //     Err(error) => {
        //         self.context
        //             .bot
        //             .send_message(self.context.msg.chat.id, error.to_string())
        //             .await?;
        //     }
        // }

        Ok(())
    }
}
