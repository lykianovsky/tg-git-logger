use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::dialogues::registration::{
    TelegramBotDialogueRegistrationDispatcher, TelegramBotDialogueRegistrationState,
};
use crate::delivery::bot::telegram::dialogues::{
    TelegramBotDialogueState, TelegramBotDialogueType,
};
use crate::delivery::bot::telegram::keyboards::actions::date_range::TelegramBotDateRangeAction;
use crate::delivery::bot::telegram::keyboards::actions::for_who::TelegramBotForWhoAction;
use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use crate::delivery::bot::telegram::keyboards::builder::KeyboardBuilder;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::error::Error;
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{DpHandlerDescription, HandlerExt, UpdateFilterExt};
use teloxide::dptree::{case, Handler};
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::{Dialogue, Requester, Update};
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup, ParseMode};
use teloxide::{dptree, Bot};

#[derive(Debug, Clone, Default)]
pub enum TelegramBotDialogueReportByDateRangeState {
    #[default]
    For,

    DateRange {
        for_who_action: TelegramBotForWhoAction,
    },
}

pub struct TelegramBotDialogueReportByDateRangeDispatcher {}

impl TelegramBotDialogueReportByDateRangeDispatcher {
    pub fn new() -> Handler<'static, Result<(), Box<dyn Error + Send + Sync>>, DpHandlerDescription>
    {
        let queries = Update::filter_callback_query()
            .enter_dialogue::<CallbackQuery, InMemStorage<TelegramBotDialogueState>, TelegramBotDialogueState>()
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::For]
                    .endpoint(TelegramBotDialogueReportByDateRangeDispatcher::choose_for_who)
            )
            .branch(
                case![TelegramBotDialogueReportByDateRangeState::DateRange { for_who_action }]
                    .endpoint(TelegramBotDialogueReportByDateRangeDispatcher::create_report_by_date_range)
            );

        dptree::entry().branch(queries)
    }

    async fn choose_for_who(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id).await?;

        let callback_data = query.data.as_deref().unwrap_or("");

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
            .update(TelegramBotDialogueState::ReportByDateRange(
                TelegramBotDialogueReportByDateRangeState::DateRange { for_who_action },
            ))
            .await?;

        let msg = query.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(chat_id, message_id, "📊 Выберите диапазон")
            .reply_markup(keyboard)
            .await?;

        Ok(())
    }

    async fn create_report_by_date_range(
        bot: Bot,
        dialogue: TelegramBotDialogueType,
        for_who_action: TelegramBotForWhoAction,
        executors: Arc<ApplicationBoostrapExecutors>,
        query: CallbackQuery,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        bot.answer_callback_query(query.id).await?;

        let callback_data = query.data.as_deref().unwrap_or("");

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

        let msg = query.message.unwrap();
        let chat_id = msg.chat().id;
        let message_id = msg.id();

        bot.edit_message_text(chat_id, message_id, "Загружаем отчёт...")
            .reply_markup(InlineKeyboardMarkup::default())
            .await?;

        let cmd = BuildVersionControlDateRangeReportExecutorCommand {
            social_user_id: SocialUserId(query.from.id.0 as i32),
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
