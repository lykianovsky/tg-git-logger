use std::fmt;

#[derive(Clone, Default, Debug)]
pub struct MessageBuilder {
    parts: Vec<String>,
    max_length: Option<usize>,
    escape_html: bool,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            max_length: None,
            escape_html: false,
        }
    }

    // Основные методы
    pub fn line(mut self, text: &str) -> Self {
        self.parts.push(text.to_string());
        self
    }

    pub fn bold(mut self, text: &str) -> Self {
        self.parts.push(format!(
            "<b>{}</b>",
            Self::escape_if_needed(text, self.escape_html)
        ));
        self
    }

    pub fn italic(mut self, text: &str) -> Self {
        self.parts.push(format!(
            "<i>{}</i>",
            Self::escape_if_needed(text, self.escape_html)
        ));
        self
    }

    pub fn code(mut self, text: &str) -> Self {
        self.parts.push(format!(
            "<code>{}</code>",
            Self::escape_if_needed(text, self.escape_html)
        ));
        self
    }

    pub fn link(mut self, text: &str, url: &str) -> Self {
        self.parts.push(format!("<a href=\"{}\">{}</a>", url, text));
        self
    }

    pub fn emoji(mut self, emoji: &str) -> Self {
        self.parts.push(emoji.to_string());
        self
    }

    pub fn empty_line(mut self) -> Self {
        self.parts.push("".to_string());
        self
    }

    // Секции с заголовком
    pub fn section(mut self, title: &str, content: &str) -> Self {
        self.parts.push(format!("<b>{}:</b> {}", title, content));
        self
    }

    pub fn section_bold(mut self, title: &str, content: &str) -> Self {
        self.parts
            .push(format!("<b>{}:</b> <b>{}</b>", title, content));
        self
    }

    pub fn section_code(mut self, title: &str, content: &str) -> Self {
        self.parts
            .push(format!("<b>{}:</b> <code>{}</code>", title, content));
        self
    }

    // Конфигурация
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    pub fn with_html_escape(mut self, escape: bool) -> Self {
        self.escape_html = escape;
        self
    }

    // Строим строку
    pub fn build(self) -> String {
        let mut result = self.parts.join("\n");

        // Обрезаем если нужно
        if let Some(max_length) = self.max_length {
            if result.len() > max_length {
                result = result.chars().take(max_length - 3).collect::<String>();
                result.push_str("...");
            }
        }

        result
    }

    // Вспомогательные методы
    fn escape_if_needed(text: &str, escape: bool) -> String {
        if escape {
            Self::escape_html(text)
        } else {
            text.to_string()
        }
    }

    fn escape_html(text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    // Метод для добавления уже отформатированной строки
    pub fn raw(mut self, text: &str) -> Self {
        self.parts.push(text.to_string());
        self
    }
}

// Реализация Display для автоматического преобразования в строку
impl fmt::Display for MessageBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.clone().build())
    }
}

// Into<String> для удобного использования
impl Into<String> for MessageBuilder {
    fn into(self) -> String {
        self.build()
    }
}

// From<&str> для создания из строки
impl From<&str> for MessageBuilder {
    fn from(text: &str) -> Self {
        MessageBuilder::new().line(text)
    }
}

// From<String> для создания из строки
impl From<String> for MessageBuilder {
    fn from(text: String) -> Self {
        MessageBuilder::new().line(&text)
    }
}