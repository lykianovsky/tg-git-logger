use crate::application::version_control::queries::build_report::command::{
    BuildVersionControlDateRangeReportExecutorCommand,
    BuildVersionControlDateRangeReportExecutorCommandForWho,
};
use crate::application::version_control::queries::build_report::error::BuildVersionControlDateRangeReportExecutorError;
use crate::application::version_control::queries::build_report::renderer;
use crate::application::version_control::queries::build_report::response::BuildVersionControlDateRangeReportExecutorResponse;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::repository::repositories::repository_task_tracker_repository::RepositoryTaskTrackerRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::services::task_tracker_service::TaskTrackerService;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::version_control::ports::version_control_client::{
    VersionControlClient, VersionControlClientDateRangeReportError,
};
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use rust_i18n::t;
use sha2::{Digest, Sha256};
use std::sync::Arc;

const REPORT_CACHE_TTL_SECONDS: u64 = 3_600; // 1 hour

pub struct BuildVersionControlDateRangeReportExecutor {
    reversible_cipher: Arc<ReversibleCipher>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_version_control_service_repo: Arc<dyn UserVersionControlAccountsRepository>,
    version_control_client: Arc<dyn VersionControlClient>,
    repository_repo: Arc<dyn RepositoryRepository>,
    repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    task_tracker_service: Arc<dyn TaskTrackerService>,
    kaiten_base: String,
    base_url: String,
    cache: Arc<dyn CacheService>,
}

impl BuildVersionControlDateRangeReportExecutor {
    pub fn new(
        reversible_cipher: Arc<ReversibleCipher>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_version_control_service_repo: Arc<dyn UserVersionControlAccountsRepository>,
        version_control_client: Arc<dyn VersionControlClient>,
        repository_repo: Arc<dyn RepositoryRepository>,
        repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
        task_tracker_service: Arc<dyn TaskTrackerService>,
        kaiten_base: String,
        base_url: String,
        cache: Arc<dyn CacheService>,
    ) -> Self {
        Self {
            reversible_cipher,
            user_socials_repo,
            user_version_control_service_repo,
            version_control_client,
            repository_repo,
            repository_task_tracker_repo,
            task_tracker_service,
            kaiten_base,
            base_url,
            cache,
        }
    }

    pub fn friendly_error_message(
        &self,
        error: &BuildVersionControlDateRangeReportExecutorError,
    ) -> String {
        tracing::error!("{error}");

        match error {
            BuildVersionControlDateRangeReportExecutorError::VersionControlClientDateRangeReportError(
                VersionControlClientDateRangeReportError::Unauthorized(reason),
            ) => t!("report.errors.unauthorized", reason = reason).to_string(),

            BuildVersionControlDateRangeReportExecutorError::VersionControlClientDateRangeReportError(
                VersionControlClientDateRangeReportError::Transport(reason),
            ) => t!("report.errors.transport", reason = reason).to_string(),

            BuildVersionControlDateRangeReportExecutorError::VersionControlClientDateRangeReportError(
                VersionControlClientDateRangeReportError::BranchNotFound(branch),
            ) => t!(
                "report.errors.branch_not_found",
                branch = MessageBuilder::escape_html(branch)
            )
            .to_string(),

            BuildVersionControlDateRangeReportExecutorError::FindSocialServiceByIdError(..) => {
                t!("report.errors.not_registered").to_string()
            }

            BuildVersionControlDateRangeReportExecutorError::BaseUrlNotConfigured => {
                t!("report.errors.not_configured").to_string()
            }

            _ => t!("report.errors.unknown").to_string(),
        }
    }

    fn compute_report_hash(
        &self,
        cmd: &BuildVersionControlDateRangeReportExecutorCommand,
    ) -> Result<String, BuildVersionControlDateRangeReportExecutorError> {
        let mut normalized = cmd.clone();
        normalized.date_range = normalized.date_range.normalize_to_day();

        let json = serde_json::to_string(&normalized)?;
        Ok(format!("{:x}", Sha256::digest(json.as_bytes())))
    }
}

impl CommandExecutor for BuildVersionControlDateRangeReportExecutor {
    type Command = BuildVersionControlDateRangeReportExecutorCommand;
    type Response = BuildVersionControlDateRangeReportExecutorResponse;
    type Error = BuildVersionControlDateRangeReportExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        if self.base_url.is_empty() {
            return Err(BuildVersionControlDateRangeReportExecutorError::BaseUrlNotConfigured);
        }

        let hash = self.compute_report_hash(cmd)?;
        let cache_key = format!("user_report_html:{}", hash);
        let report_url = format!("{}/report/{}", self.base_url, hash);

        let is_cached = self.cache.get(&cache_key).await.ok().flatten().is_some();

        if is_cached {
            tracing::debug!(hash = %hash, "Report HTML cache hit — returning existing URL");
            return Ok(BuildVersionControlDateRangeReportExecutorResponse { report_url });
        }

        // ── Cache miss: fetch fresh data and render ───────────────────────────

        let social_user = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;

        let version_control_user = self
            .user_version_control_service_repo
            .find_by_user_id(&social_user.user_id)
            .await?;

        let decrypted_token = self
            .reversible_cipher
            .decrypt(version_control_user.access_token.value())?;

        let repository = self.repository_repo.find_by_id(cmd.repository_id).await?;

        let author = match cmd.for_who {
            BuildVersionControlDateRangeReportExecutorCommandForWho::Me => {
                Some(version_control_user.version_control_login)
            }
            BuildVersionControlDateRangeReportExecutorCommandForWho::Repository => None,
        };

        let report = self
            .version_control_client
            .get_details_by_range(
                &decrypted_token,
                &repository.owner,
                &repository.name,
                &cmd.branch,
                &cmd.date_range,
                author.as_deref(),
            )
            .await?;

        tracing::debug!(
            repo = %repository.name,
            commits = report.commits.len(),
            prs = report.pull_requests.len(),
            "Version control report fetched"
        );

        let tracker = self
            .repository_task_tracker_repo
            .find_by_repository_id(repository.id)
            .await
            .ok();

        // ── Render HTML via template ──────────────────────────────────────────

        let html = renderer::build_html_report(
            &report,
            &author,
            &cmd.date_range,
            &repository.owner,
            &repository.name,
            &cmd.branch,
            tracker.as_ref(),
            self.task_tracker_service.as_ref(),
            &self.kaiten_base,
        )
        .map_err(|e| {
            BuildVersionControlDateRangeReportExecutorError::TemplateRender(e.to_string())
        })?;

        // ── Cache and return ──────────────────────────────────────────────────

        self.cache
            .set(&cache_key, &html, REPORT_CACHE_TTL_SECONDS)
            .await
            .map_err(|e| BuildVersionControlDateRangeReportExecutorError::Cache(e.to_string()))?;

        Ok(BuildVersionControlDateRangeReportExecutorResponse { report_url })
    }
}
