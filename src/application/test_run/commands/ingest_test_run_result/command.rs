use crate::domain::test_run::value_objects::run_tag::RunTag;
use chrono::{DateTime, Utc};

/// Что вебхук уже рассказал о прогоне. Если это есть, статус в CI не переспрашиваем
#[derive(Debug, Clone)]
pub struct KnownTestRunState {
    pub provider_run_id: u64,
    /// Статус прогона в CI: queued / in_progress / completed
    pub status: String,
    /// Итог завершённого прогона: success / failure / cancelled …
    pub conclusion: Option<String>,
    pub run_url: Option<String>,
    pub sha: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

/// Прогон завершился в CI: метку берём из имени прогона в вебхуке
pub struct IngestTestRunResultCommand {
    pub run_tag: RunTag,
    /// Данные из вебхука; без них статус добирается запросом в CI
    pub known_state: Option<KnownTestRunState>,
}
