use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::report::TelegramBotDialogueReportByDateRangeState;
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use std::sync::Arc;
use teloxide::dispatching::dialogue::Storage;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;

pub struct TelegramBotVersionControlReportCommandHandler {
    context: TelegramBotCommandContext,
    dialogue: Arc<TelegramBotDialogueType>,
}

impl TelegramBotVersionControlReportCommandHandler {
    pub fn new(context: TelegramBotCommandContext, dialogue: Arc<TelegramBotDialogueType>) -> Self {
        Self { context, dialogue }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotForWhoAction>(vec![
                TelegramBotForWhoAction::Me,
                TelegramBotForWhoAction::Repository,
            ])
            .build();

        self.dialogue
            .update(TelegramBotDialogueState::ReportByDateRange(
                TelegramBotDialogueReportByDateRangeState::For,
            ))
            .await?;

        self.context
            .bot
            .send_message(self.context.msg.chat.id, "🎯  Выберите для кого:")
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }
}
