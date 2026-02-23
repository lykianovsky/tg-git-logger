use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::delivery::bot::telegram::context::TelegramBotCommandContext;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Requester;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, Message};
use teloxide::RequestError;

pub struct TelegramBotWeeklyReportCommandHandler {
    context: TelegramBotCommandContext,
    executor: Arc<BuildVersionControlDateRangeReportExecutor>,
}

impl TelegramBotWeeklyReportCommandHandler {
    pub fn new(
        context: TelegramBotCommandContext,
        executor: Arc<BuildVersionControlDateRangeReportExecutor>,
    ) -> Self {
        Self { context, executor }
    }

    pub async fn execute(&self) -> Result<Message, RequestError> {
        let keyboard = InlineKeyboardMarkup::new(vec![
            vec![
                InlineKeyboardButton::callback("📅 Last week", "weekly_report:last_week"),
                InlineKeyboardButton::callback("📅 Last 2 weeks", "weekly_report:last_2_weeks"),
            ],
            vec![
                InlineKeyboardButton::callback("📅 Last month", "weekly_report:last_month"),
                InlineKeyboardButton::callback("📅 This month", "weekly_report:this_month"),
            ],
        ]);

        self.context
            .bot
            .send_message(self.context.msg.chat.id, "📊 Выберите период:")
            .reply_markup(keyboard)
            .await
    }
}
