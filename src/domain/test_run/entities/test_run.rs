use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use crate::domain::test_run::value_objects::test_run_trigger::TestRunTrigger;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};

/// Итоги прогона: заполняются из машиночитаемого отчёта после завершения
#[derive(Debug, Clone, Copy, Default)]
pub struct TestRunTotals {
    pub total: u32,
    pub passed: u32,
    pub failed: u32,
    pub flaky: u32,
    pub skipped: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TestRun {
    pub id: TestRunId,
    pub repository_id: RepositoryId,
    pub run_tag: RunTag,
    pub provider_run_id: Option<u64>,
    pub run_url: Option<String>,
    pub git_ref: String,
    pub sha: Option<String>,
    pub trigger: TestRunTrigger,
    pub args: Option<String>,
    pub requested_by_user_id: Option<UserId>,
    /// Карточка прогона в чате: её же сообщение обновляется по ходу
    pub chat_id: Option<SocialChatId>,
    pub message_id: Option<i32>,
    pub status: TestRunStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub totals: Option<TestRunTotals>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewTestRun {
    pub repository_id: RepositoryId,
    pub run_tag: RunTag,
    pub git_ref: String,
    pub trigger: TestRunTrigger,
    pub args: Option<String>,
    pub requested_by_user_id: Option<UserId>,
    pub chat_id: Option<SocialChatId>,
}

/// Что известно о прогоне после его завершения в CI
#[derive(Debug, Clone)]
pub struct TestRunOutcome {
    pub status: TestRunStatus,
    pub provider_run_id: Option<u64>,
    pub run_url: Option<String>,
    pub sha: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub totals: Option<TestRunTotals>,
}

impl TestRun {
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn has_failures(&self) -> bool {
        self.totals.is_some_and(|totals| totals.failed > 0)
    }
}
