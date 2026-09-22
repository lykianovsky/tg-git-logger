#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRunStatus {
    /// Запуск отправлен в CI, прогон ещё не стартовал
    Queued,
    Running,
    Passed,
    Failed,
    Cancelled,
    /// Прогон завершился, но итогов в артефакте нет
    Unknown,
}

impl TestRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "passed" => Some(Self::Passed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running)
    }
}
