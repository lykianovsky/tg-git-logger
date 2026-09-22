use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_failure_card::{NewTestFailureCard, TestFailureCard};
use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FindTestFailureCardError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum SaveTestFailureCardError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait TestFailureCardRepository: Send + Sync {
    async fn find_by_fingerprint(
        &self,
        repository_id: RepositoryId,
        fingerprint: &TestFingerprint,
    ) -> Result<Option<TestFailureCard>, FindTestFailureCardError>;

    /// Карточка на тест одна: повторное заведение перезаписывает запись, сохраняя ссылку
    /// на предыдущую карточку
    async fn upsert(
        &self,
        card: &NewTestFailureCard,
    ) -> Result<TestFailureCard, SaveTestFailureCardError>;

    async fn mark_closed(
        &self,
        id: i32,
        closed_at: DateTime<Utc>,
    ) -> Result<(), SaveTestFailureCardError>;
}
