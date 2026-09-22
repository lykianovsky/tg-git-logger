use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreTestReportError {
    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(Debug, Error)]
pub enum BuildTestReportLinkError {
    #[error("Report is not stored")]
    NotStored,

    #[error("Storage error: {0}")]
    StorageError(String),
}

#[async_trait::async_trait]
pub trait TestReportStorage: Send + Sync {
    /// Кладёт распакованный HTML-отчёт под прогон и возвращает занятый объём
    async fn store(&self, id: TestRunId, source_dir: &str) -> Result<u64, StoreTestReportError>;

    /// Ссылка со сроком жизни: открывается без авторизации в CI
    async fn build_link(&self, id: TestRunId) -> Result<String, BuildTestReportLinkError>;

    async fn remove_expired(&self) -> Result<u32, StoreTestReportError>;
}
