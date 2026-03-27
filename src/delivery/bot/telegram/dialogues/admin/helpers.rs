use teloxide::prelude::Message;

/// Извлекает текстовое содержимое сообщения, обрезая пробелы.
/// Возвращает None если сообщение не содержит текста или текст пустой.
pub fn extract_text(msg: &Message) -> Option<String> {
    msg.text()
        .map(|t| t.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Парсит целое число из текстового сообщения.
pub fn parse_integer(msg: &Message) -> Option<i32> {
    msg.text().and_then(|t| t.trim().parse().ok())
}

/// Дружелюбное сообщение об ошибке операции с БД.
pub fn db_error_message(action: &str) -> String {
    format!("❌ Не удалось {}. Попробуйте позже.", action)
}
