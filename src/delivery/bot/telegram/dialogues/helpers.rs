use crate::delivery::bot::telegram::keyboards::actions::TelegramBotKeyboardAction;
use teloxide::payloads::EditMessageTextSetters;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardMarkup, MessageId, ParseMode};

/// Extracts the text content from a message, trimmed.
/// Returns None if the message has no text or is empty.
pub fn extract_text(msg: &Message) -> Option<String> {
    msg.text()
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Parses an integer from a text message.
pub fn parse_integer(msg: &Message) -> Option<i32> {
    msg.text().and_then(|t| t.trim().parse().ok())
}

/// Result of parsing a callback query: the parsed action and the
/// original message (for chat_id / message_id).
pub struct CallbackContext<A> {
    pub action: A,
    pub chat_id: ChatId,
    pub message_id: MessageId,
}

/// Acknowledges a callback query and parses the action + message from it.
///
/// Returns `Ok(None)` when the query is malformed (no data / no message)
/// — caller should just `return Ok(())` in that case.
pub async fn parse_callback<A: TelegramBotKeyboardAction>(
    bot: &Bot,
    query: &CallbackQuery,
) -> Result<Option<CallbackContext<A>>, Box<dyn std::error::Error + Send + Sync>> {
    bot.answer_callback_query(query.id.clone()).await?;

    let data = match query.data.as_deref() {
        Some(d) => d,
        None => return Ok(None),
    };

    let action = match A::from_callback_data(data) {
        Ok(a) => a,
        Err(e) => {
            tracing::error!(error = %e, data = %data, "Unknown callback action");
            return Ok(None);
        }
    };

    let msg = match &query.message {
        Some(m) => m,
        None => return Ok(None),
    };

    Ok(Some(CallbackContext {
        action,
        chat_id: msg.chat().id,
        message_id: msg.id(),
    }))
}

/// Редактирует существующее сообщение (для навигации между submenu).
/// Если edit фейлит (старое сообщение / нет изменений / API limit) — шлёт новое.
pub async fn edit_menu(
    bot: &Bot,
    chat_id: ChatId,
    message_id: MessageId,
    text: &str,
    keyboard: Option<InlineKeyboardMarkup>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let edit_result = match keyboard.clone() {
        Some(kb) => {
            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .reply_markup(kb)
                .await
        }
        None => {
            bot.edit_message_text(chat_id, message_id, text)
                .parse_mode(ParseMode::Html)
                .await
        }
    };

    if let Err(e) = edit_result {
        tracing::debug!(error = %e, "Edit message failed, sending new instead");
        match keyboard {
            Some(kb) => {
                bot.send_message(chat_id, text)
                    .parse_mode(ParseMode::Html)
                    .reply_markup(kb)
                    .await?;
            }
            None => {
                bot.send_message(chat_id, text)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
    }
    Ok(())
}

/// Удаляет сообщение с меню (для "Закрыть"). Ошибки игнорируются.
pub async fn close_menu(bot: &Bot, chat_id: ChatId, message_id: MessageId) {
    let _ = bot.delete_message(chat_id, message_id).await;
}
