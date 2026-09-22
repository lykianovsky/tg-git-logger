use crate::domain::test_run::ports::test_report_storage::{
    BuildTestReportLinkError, StoreTestReportError, TestReportStorage,
};
use crate::domain::test_run::value_objects::test_run_id::TestRunId;
use crate::infrastructure::drivers::cache::contract::CacheService;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

/// Префикс ключа кэша со ссылкой на отчёт: по токену из ссылки находим прогон
pub const TEST_REPORT_TOKEN_CACHE_PREFIX: &str = "test_report_token:";

const TOKEN_BYTES: usize = 16;
const SECONDS_IN_MINUTE: u64 = 60;
const SECONDS_IN_DAY: u64 = 86_400;

/// Отчёт Playwright — каталог файлов, поэтому бот держит его на диске, а не в кэше:
/// в кэше живёт только короткоживущий токен ссылки.
pub struct LocalTestReportStorage {
    reports_dir: PathBuf,
    base_url: String,
    link_ttl_minutes: i64,
    retention_days: i64,
    cache: Arc<dyn CacheService>,
}

impl LocalTestReportStorage {
    pub fn new(
        reports_dir: String,
        base_url: String,
        link_ttl_minutes: i64,
        retention_days: i64,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self {
            reports_dir: PathBuf::from(reports_dir),
            base_url,
            link_ttl_minutes,
            retention_days,
            cache,
        }
    }

    pub fn run_dir(&self, id: TestRunId) -> PathBuf {
        self.reports_dir.join(id.0.to_string())
    }

    fn generate_token() -> String {
        use rand::RngCore;

        let mut random = [0u8; TOKEN_BYTES];

        rand::rngs::OsRng.fill_bytes(&mut random);

        hex::encode(random)
    }

    /// Каталог копируем целиком: артефакт распакован во временное место, которое уйдёт
    async fn copy_dir(source: PathBuf, target: PathBuf) -> Result<u64, std::io::Error> {
        tokio::task::spawn_blocking(move || copy_dir_blocking(&source, &target))
            .await
            .map_err(std::io::Error::other)?
    }
}

fn copy_dir_blocking(source: &Path, target: &Path) -> Result<u64, std::io::Error> {
    std::fs::create_dir_all(target)?;

    let mut copied = 0;

    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let entry_target = target.join(entry.file_name());

        if entry.file_type()?.is_dir() {
            copied += copy_dir_blocking(&entry.path(), &entry_target)?;
            continue;
        }

        copied += std::fs::copy(entry.path(), entry_target)?;
    }

    Ok(copied)
}

fn remove_expired_blocking(reports_dir: &Path, max_age_secs: u64) -> Result<u32, std::io::Error> {
    if !reports_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0;

    for entry in std::fs::read_dir(reports_dir)? {
        let entry = entry?;

        if !entry.file_type()?.is_dir() {
            continue;
        }

        let modified = entry.metadata()?.modified()?;
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default()
            .as_secs();

        if age < max_age_secs {
            continue;
        }

        std::fs::remove_dir_all(entry.path())?;

        removed += 1;
    }

    Ok(removed)
}

#[async_trait]
impl TestReportStorage for LocalTestReportStorage {
    async fn store(&self, id: TestRunId, source_dir: &str) -> Result<u64, StoreTestReportError> {
        let target = self.run_dir(id);

        // Повторный вебхук того же прогона перезаписывает отчёт, а не смешивается с ним
        if target.exists() {
            tokio::fs::remove_dir_all(&target)
                .await
                .map_err(|error| StoreTestReportError::StorageError(error.to_string()))?;
        }

        Self::copy_dir(PathBuf::from(source_dir), target)
            .await
            .map_err(|error| StoreTestReportError::StorageError(error.to_string()))
    }

    async fn build_link(&self, id: TestRunId) -> Result<String, BuildTestReportLinkError> {
        if self.base_url.is_empty() {
            return Err(BuildTestReportLinkError::StorageError(
                "APPLICATION_BASE_URL is not set".to_string(),
            ));
        }

        if !self.run_dir(id).join("index.html").exists() {
            return Err(BuildTestReportLinkError::NotStored);
        }

        let token = Self::generate_token();
        let ttl_secs = self.link_ttl_minutes.max(1) as u64 * SECONDS_IN_MINUTE;

        self.cache
            .set(
                &format!("{TEST_REPORT_TOKEN_CACHE_PREFIX}{token}"),
                &id.0.to_string(),
                ttl_secs,
            )
            .await
            .map_err(BuildTestReportLinkError::StorageError)?;

        Ok(format!("{}/test-report/{}/", self.base_url, token))
    }

    async fn remove_expired(&self) -> Result<u32, StoreTestReportError> {
        let reports_dir = self.reports_dir.clone();
        let max_age_secs = self.retention_days.max(1) as u64 * SECONDS_IN_DAY;

        tokio::task::spawn_blocking(move || remove_expired_blocking(&reports_dir, max_age_secs))
            .await
            .map_err(|error| StoreTestReportError::StorageError(error.to_string()))?
            .map_err(|error| StoreTestReportError::StorageError(error.to_string()))
    }
}
