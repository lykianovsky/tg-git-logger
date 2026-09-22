use crate::application::test_run::queries::build_test_report::error::BuildTestReportError;
use crate::application::test_run::queries::build_test_report::query::BuildTestReportQuery;
use crate::application::test_run::queries::build_test_report::renderer;
use crate::application::test_run::queries::build_test_report::response::BuildTestReportResponse;
use crate::application::test_run::queries::get_run_failures::executor::GetRunFailuresExecutor;
use crate::application::test_run::queries::get_run_failures::query::GetRunFailuresQuery;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::infrastructure::drivers::cache::contract::CacheService;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

/// Столько живёт отрисованная страница отчёта — как у отчётов по репозиторию
const REPORT_CACHE_TTL_SECONDS: u64 = 3_600;

pub struct BuildTestReportExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
    get_run_failures: Arc<GetRunFailuresExecutor>,
    base_url: String,
    cache: Arc<dyn CacheService>,
    report_url_secret: String,
}

impl BuildTestReportExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
        get_run_failures: Arc<GetRunFailuresExecutor>,
        base_url: String,
        cache: Arc<dyn CacheService>,
        report_url_secret: String,
    ) -> Self {
        Self {
            test_run_repo,
            repository_repo,
            get_run_failures,
            base_url,
            cache,
            report_url_secret,
        }
    }

    /// Адрес отчёта не угадывается: он подписан тем же секретом, что и отчёты по репозиторию
    fn compute_report_hash(&self, run_tag: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.report_url_secret.as_bytes())
            .expect("HMAC accepts any-length key");

        mac.update(run_tag.as_bytes());

        format!("{:x}", mac.finalize().into_bytes())
    }
}

impl CommandExecutor for BuildTestReportExecutor {
    type Command = BuildTestReportQuery;
    type Response = BuildTestReportResponse;
    type Error = BuildTestReportError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        if self.base_url.is_empty() {
            return Err(BuildTestReportError::BaseUrlNotConfigured);
        }

        let run = self.test_run_repo.find_by_id(query.test_run_id).await?;

        let hash = self.compute_report_hash(run.run_tag.as_str());
        let cache_key = format!("user_report_html:{hash}");
        let report_url = format!("{}/report/{}", self.base_url, hash);

        // Прогон уже завершён, его итоги не меняются — отрисованную страницу переиспользуем
        if self.cache.get(&cache_key).await.ok().flatten().is_some() {
            return Ok(BuildTestReportResponse { report_url });
        }

        let repository = self.repository_repo.find_by_id(run.repository_id).await?;
        let failures = self
            .get_run_failures
            .execute(&GetRunFailuresQuery {
                test_run_id: query.test_run_id,
            })
            .await?;

        let html = renderer::build_html_report(&run, &repository, &failures.failures)?;

        self.cache
            .set(&cache_key, &html, REPORT_CACHE_TTL_SECONDS)
            .await
            .map_err(BuildTestReportError::DbError)?;

        Ok(BuildTestReportResponse { report_url })
    }
}
