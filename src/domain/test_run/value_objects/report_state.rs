#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestReportState {
    /// Отчёта у бота нет: прогон не дошёл до тестов или артефакт пуст
    None,
    Stored,
    /// Отчёт больше допустимого размера — не сохраняли
    TooLarge,
    /// Файлы удалены по сроку хранения
    Expired,
}

impl TestReportState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Stored => "stored",
            Self::TooLarge => "too_large",
            Self::Expired => "expired",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "stored" => Some(Self::Stored),
            "too_large" => Some(Self::TooLarge),
            "expired" => Some(Self::Expired),
            _ => None,
        }
    }
}
