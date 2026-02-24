use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use crate::delivery::bot::telegram::dialogues::report::{
    ReportByDateRangeDialogue, TelegramBotReportByDateRangeDialogueState,
};
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::Message;
use teloxide::RequestError;

pub struct TelegramBotVersionControlReportCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<BuildVersionControlDateRangeReportExecutor>,
    dialog: Arc<ReportByDateRangeDialogue>,
}

impl TelegramBotVersionControlReportCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executor: Arc<BuildVersionControlDateRangeReportExecutor>,
        dialog: Arc<ReportByDateRangeDialogue>,
    ) -> Self {
        Self {
            context,
            executor,
            dialog,
        }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotForWhoAction>(vec![
                TelegramBotForWhoAction::Me,
                TelegramBotForWhoAction::Repository,
            ])
            .build();

        self.dialog
            .update(TelegramBotReportByDateRangeDialogueState::For)
            .await
            .expect("failed to update dialog2ue");

        // Сразу отправляем клавиатуру
        self.context
            .bot
            .send_message(self.context.msg.chat.id, "🎯  Выберите для кого:")
            .reply_markup(keyboard)
            .await
    }
}
