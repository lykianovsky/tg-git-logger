use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::application::version_control::queries::build_report::error::BuildVersionControlDateRangeReportExecutorError;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::keyboards::actions::date_range::TelegramBotDateRangeAction;
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::version_control::ports::version_control_client::VersionControlClientDateRangeReportError;
use std::fmt::format;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{Dialogue, Requester, ResponseResult};
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup, ParseMode};
use teloxide::{Bot, RequestError};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotReportByDateRangeDialogueState {
    #[default]
    For,

    DateRange {
        for_who_action: TelegramBotForWhoAction,
    },
}

pub type ReportByDateRangeDialogue = Dialogue<
    TelegramBotReportByDateRangeDialogueState,
    InMemStorage<TelegramBotReportByDateRangeDialogueState>,
>;

pub struct TelegramBotReportByDateRangeDialogue {}

impl TelegramBotReportByDateRangeDialogue {
    pub async fn choose_for_who(
        bot: Bot,
        dialogue: ReportByDateRangeDialogue,
        q: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(q.id).await?;

        let callback_data = q.data.as_deref().unwrap_or("");

        let for_who_action = match TelegramBotForWhoAction::from_callback_data(callback_data) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{e}");
                return Ok(());
            }
        };

        let keyboard = KeyboardBuilder::new()
            .row::<TelegramBotDateRangeAction>(vec![
                TelegramBotDateRangeAction::LastWeek,
                TelegramBotDateRangeAction::Last2Weeks,
            ])
            .row::<TelegramBotDateRangeAction>(vec![
                TelegramBotDateRangeAction::LastMonth,
                TelegramBotDateRangeAction::ThisMonth,
            ])
            .build();

        dialogue
            .update(TelegramBotReportByDateRangeDialogueState::DateRange { for_who_action })
            .await?;

        let msg = q.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(chat_id, message_id, "📊 Выберите диапазон")
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    pub async fn create_report_by_date_range(
        bot: Bot,
        dialogue: ReportByDateRangeDialogue,
        for_who_action: TelegramBotForWhoAction,
        executors: Arc<ApplicationBoostrapExecutors>,
        q: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(q.id).await?;

        let callback_data = q.data.as_deref().unwrap_or("");

        let date_range_action = match TelegramBotDateRangeAction::from_callback_data(callback_data)
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{e}");
                return Ok(());
            }
        };

        let date_range = match date_range_action {
            TelegramBotDateRangeAction::LastWeek => DateRange::last_week(),
            TelegramBotDateRangeAction::Last2Weeks => DateRange::last_2_weeks(),
            TelegramBotDateRangeAction::LastMonth => DateRange::last_month(),
            TelegramBotDateRangeAction::ThisMonth => DateRange::this_month(),
        };

        let for_who = match for_who_action {
            TelegramBotForWhoAction::Me => {
                BuildVersionControlDateRangeReportExecutorCommandForWho::Me
            }
            TelegramBotForWhoAction::Repository => {
                BuildVersionControlDateRangeReportExecutorCommandForWho::Repository
            }
        };

        let msg = q.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(chat_id, message_id, "Загружаем отчёт...")
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        let cmd = BuildVersionControlDateRangeReportExecutorCommand {
            social_user_id: SocialUserId(q.from.id.0 as i32),
            date_range,
            for_who,
        };

        let executor = executors.queries.build_report_by_range.clone();

        match executor.execute(&cmd).await {
            Ok(response) => {
                bot.edit_message_text(chat_id, message_id, response.text)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
            Err(error) => {
                bot.edit_message_text(
                    chat_id,
                    message_id,
                    executor.friendly_error_message(&error).to_string(),
                )
                .await?;
            }
        };

        dialogue.exit().await?;

        Ok(())
    }
}
