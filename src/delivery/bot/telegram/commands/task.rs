use crate::application::task::queries::get_task_card::error::GetTaskCardError;
use crate::application::task::queries::get_task_card::executor::GetTaskCardExecutor;
use crate::application::task::queries::get_task_card::query::GetTaskCardQuery;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::value_objects::task_id::TaskId;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::prelude::Requester;
use teloxide::types::{Message, ParseMode};

pub struct TelegramBotTaskCommandHandler {
    bot: Bot,
    msg: Message,
    get_task_card: Arc<GetTaskCardExecutor>,
    raw_id: String,
}

impl TelegramBotTaskCommandHandler {
    pub fn new(
        bot: Bot,
        msg: Message,
        get_task_card: Arc<GetTaskCardExecutor>,
        raw_id: String,
    ) -> Self {
        Self {
            bot,
            msg,
            get_task_card,
            raw_id,
        }
    }

    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let id: u64 = match self.raw_id.trim().parse() {
            Ok(v) => v,
            Err(_) => {
                self.bot
                    .send_message(
                        self.msg.chat.id,
                        "❌ Введите числовой ID карточки: <code>/task 12345</code>",
                    )
                    .parse_mode(ParseMode::Html)
                    .await?;
                return Ok(());
            }
        };

        let loading = self
            .bot
            .send_message(self.msg.chat.id, "⏳ Ищем карточку...")
            .await?;

        let text = match self
            .get_task_card
            .execute(&GetTaskCardQuery {
                task_id: TaskId(id),
            })
            .await
        {
            Ok(card) => format!(
                "📋 <b>{}</b>\n🔗 <a href=\"{}\">Открыть карточку →</a>",
                teloxide::utils::html::escape(&card.title),
                card.url,
            ),
            Err(GetTaskCardError::NotFound) => {
                format!("❌ Карточка <code>{}</code> не найдена.", id)
            }
            Err(GetTaskCardError::ClientError(e)) => {
                tracing::error!(error = %e, task_id = id, "Failed to fetch task card");
                "❌ Ошибка при получении карточки. Попробуйте позже.".to_string()
            }
        };

        self.bot
            .edit_message_text(self.msg.chat.id, loading.id, text)
            .parse_mode(ParseMode::Html)
            .await?;

        Ok(())
    }
}
