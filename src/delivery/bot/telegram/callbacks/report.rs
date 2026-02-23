use crate::application::version_control::queries::build_report::command::BuildVersionControlDateRangeReportExecutorCommand;
use crate::application::version_control::queries::build_report::executor::BuildVersionControlDateRangeReportExecutor;
use crate::domain::shared::date::range::DateRange;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use std::sync::Arc;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::prelude::Requester;
use teloxide::types::{CallbackQuery, ParseMode};
use teloxide::{Bot, RequestError};

pub struct TelegramBotReportCallbackHandler {
    bot: Bot,
    query: CallbackQuery,
    executor: Arc<BuildVersionControlDateRangeReportExecutor>,
}

impl TelegramBotReportCallbackHandler {
    pub fn new(
        bot: Bot,
        query: CallbackQuery,
        executor: Arc<BuildVersionControlDateRangeReportExecutor>,
    ) -> Self {
        Self {
            bot,
            query,
            executor,
        }
    }

    pub async fn execute(&self) -> Result<(), RequestError> {
        let chat_id = self.query.message.as_ref().unwrap().chat().id;

        let data = match self.query.data.as_deref() {
            Some(d) => d,
            None => return Ok(()),
        };

        let range = match data {
            "weekly_report:last_week" => DateRange::last_week(),
            "weekly_report:last_2_weeks" => DateRange::last_2_weeks(),
            "weekly_report:last_month" => DateRange::last_month(),
            "weekly_report:this_month" => DateRange::this_month(),
            _ => return Ok(()),
        };

        self.bot
            .answer_callback_query(self.query.id.clone())
            .await?;

        let loading = self
            .bot
            .send_message(chat_id, "⏳ Building report...")
            .await?;

        let cmd = BuildVersionControlDateRangeReportExecutorCommand {
            social_user_id: SocialUserId(self.query.from.id.0 as i32),
            date_range: range,
        };

        match self.executor.execute(cmd).await {
            Ok(response) => {
                self.bot
                    .edit_message_text(chat_id, loading.id, response.text)
                    .parse_mode(ParseMode::Html)
                    .await?;
                Ok(())
            }
            Err(e) => {
                self.bot
                    .edit_message_text(chat_id, loading.id, format!("❌ {e}"))
                    .await?;
                Ok(())
            }
        }
    }
}
