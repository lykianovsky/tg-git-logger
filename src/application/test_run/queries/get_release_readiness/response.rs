use crate::domain::test_run::entities::test_failure::TestFailure;
use crate::domain::test_run::entities::test_run::TestRun;

/// Почему репозиторий не готов — формулировка для человека, а не код ошибки
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseReadinessVerdict {
    /// Последний прогон зелёный и свежий
    Ready,
    /// Есть упавшие тесты
    Failed,
    /// Прогон зелёный, но давний — о сегодняшнем состоянии он не говорит
    Stale,
    /// Прогон идёт прямо сейчас
    Running,
    /// Тесты не подключены или прогонов ещё не было
    NoData,
}

impl ReleaseReadinessVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Stale => "stale",
            Self::Running => "running",
            Self::NoData => "no_data",
        }
    }
}

pub struct GetReleaseReadinessResponse {
    pub verdict: ReleaseReadinessVerdict,
    pub run: Option<TestRun>,
    /// Что именно мешает релизу
    pub failures: Vec<TestFailure>,
}
