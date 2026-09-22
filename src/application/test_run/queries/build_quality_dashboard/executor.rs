use crate::application::test_run::queries::build_quality_dashboard::error::BuildQualityDashboardError;
use crate::application::test_run::queries::build_quality_dashboard::query::BuildQualityDashboardQuery;
use crate::application::test_run::queries::build_quality_dashboard::renderer;
use crate::application::test_run::queries::build_quality_dashboard::response::BuildQualityDashboardResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use crate::infrastructure::drivers::cache::contract::CacheService;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

/// Страница состояния живёт недолго: она о «сейчас», а не о моменте в прошлом
const DASHBOARD_CACHE_TTL_SECONDS: u64 = 300;
/// Сколько последних прогонов показываем на графике
const RUNS_LIMIT: u64 = 30;
/// За какой период считаем частоту падений
const FAILURES_PERIOD_DAYS: i64 = 30;

pub struct BuildQualityDashboardExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
    base_url: String,
    cache: Arc<dyn CacheService>,
    report_url_secret: String,
}

impl BuildQualityDashboardExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
        base_url: String,
        cache: Arc<dyn CacheService>,
        report_url_secret: String,
    ) -> Self {
        Self {
            test_run_repo,
            repository_repo,
            base_url,
            cache,
            report_url_secret,
        }
    }

    fn compute_hash(&self, repository_id: i32) -> String {
        let mut mac = HmacSha256::new_from_slice(self.report_url_secret.as_bytes())
            .expect("HMAC accepts any-length key");

        mac.update(format!("quality:{repository_id}").as_bytes());

        format!("{:x}", mac.finalize().into_bytes())
    }
}

impl CommandExecutor for BuildQualityDashboardExecutor {
    type Command = BuildQualityDashboardQuery;
    type Response = BuildQualityDashboardResponse;
    type Error = BuildQualityDashboardError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        if self.base_url.is_empty() {
            return Err(BuildQualityDashboardError::BaseUrlNotConfigured);
        }

        let hash = self.compute_hash(query.repository_id.0);
        let cache_key = format!("user_report_html:{hash}");
        let dashboard_url = format!("{}/report/{}", self.base_url, hash);

        let repository = self.repository_repo.find_by_id(query.repository_id).await?;
        let runs = self
            .test_run_repo
            .list_recent(query.repository_id, RUNS_LIMIT)
            .await?;
        let failures = self
            .test_run_repo
            .count_failures_since(
                query.repository_id,
                chrono::Utc::now() - chrono::Duration::days(FAILURES_PERIOD_DAYS),
            )
            .await?;

        let html = renderer::build_html_dashboard(&repository, &runs, &failures)?;

        self.cache
            .set(&cache_key, &html, DASHBOARD_CACHE_TTL_SECONDS)
            .await
            .map_err(BuildQualityDashboardError::DbError)?;

        Ok(BuildQualityDashboardResponse { dashboard_url })
    }
}
