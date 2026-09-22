#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRunTrigger {
    /// Запуск из чата: итог обновляет карточку, из которой запускали
    Chat,
    /// Ночной прогон по расписанию: итог приходит в чат репозитория
    Schedule,
    /// Ручной запуск в интерфейсе CI
    Manual,
}

impl TestRunTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::Schedule => "schedule",
            Self::Manual => "manual",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "chat" => Some(Self::Chat),
            "schedule" => Some(Self::Schedule),
            "manual" => Some(Self::Manual),
            _ => None,
        }
    }
}
