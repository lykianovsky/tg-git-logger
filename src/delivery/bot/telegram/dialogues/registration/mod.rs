use crate::delivery::bot::telegram::dialogues::TelegramBotDialogueType;
use crate::delivery::bot::telegram::keyboards::actions::choose_role::TelegramBotChooseRoleAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use std::error::Error;
use teloxide::dispatching::DpHandlerDescription;
use teloxide::dptree::case;
use teloxide::prelude::*;
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

        let msg = query.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(
            chat_id,
            message_id,
            format!("Выбранная роль: {}", selected_role.to_callback_data()),
        )
        .await?;

        dialogue.exit().await.ok();

        Ok(())
    }
}
