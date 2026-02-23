pub mod report;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::delivery::bot::telegram::callbacks::report::TelegramBotReportCallbackHandler;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::Bot;

pub async fn handle_callback(
    query: CallbackQuery,
    bot: Bot,
    executors: Arc<ApplicationBoostrapExecutors>,
) -> ResponseResult<()> {
    let data = match query.data.as_deref() {
        Some(d) => d,
        None => return Ok(()),
    };

    tracing::debug!("Received callback query with data: {}", data);

    match data.split(':').next() {
        Some("weekly_report") => {
            TelegramBotReportCallbackHandler::new(
                bot,
                query,
                executors.queries.build_weekly_report.clone(),
            )
            .execute()
            .await?;
        }
        _ => {
            bot.answer_callback_query(query.id).await?;
        }
    }

    Ok(())
}
