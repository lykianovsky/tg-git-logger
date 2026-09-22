use crate::domain::test_run::entities::test_run::TestRunOutcome;
use crate::domain::test_run::entities::test_suite::TestSuite;
use crate::domain::test_run::ports::test_runner::{
    DispatchTestRunError, FetchTestRunError, ListTestBlocksError, TestRunArtifacts, TestRunner,
};
use crate::domain::test_run::value_objects::report_state::TestReportState;
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use crate::infrastructure::integrations::test_runner::summary::parse_summary;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Сколько последних прогонов просматриваем, разыскивая свой по метке
const RUNS_LOOKUP_LIMIT: u32 = 30;
const GITHUB_API_VERSION: &str = "2022-11-28";
const USER_AGENT: &str = "tg-bot-logger";
const BYTES_IN_MEGABYTE: u64 = 1024 * 1024;

pub struct GithubActionsTestRunner {
    http: reqwest::Client,
    api_base: String,
    token: String,
    /// Куда распаковывается HTML-отчёт до переноса в постоянное хранилище
    work_dir: PathBuf,
    report_max_size_mb: u64,
}

impl GithubActionsTestRunner {
    pub fn new(
        api_base: String,
        token: String,
        work_dir: PathBuf,
        report_max_size_mb: u64,
    ) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base,
            token,
            work_dir,
            report_max_size_mb,
        }
    }

    fn request(&self, method: reqwest::Method, url: String) -> reqwest::RequestBuilder {
        self.http
            .request(method, url)
            .bearer_auth(&self.token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
    }

    fn repository_url(&self, suite: &TestSuite, path: &str) -> String {
        format!(
            "{}/repos/{}/{}/{}",
            self.api_base.trim_end_matches('/'),
            suite.owner,
            suite.name,
            path
        )
    }

    /// Прогон CI переводится в доменный статус: пока он идёт, conclusion пуст
    fn build_outcome(run: &Value) -> TestRunOutcome {
        let status = run.get("status").and_then(Value::as_str).unwrap_or("");
        let conclusion = run.get("conclusion").and_then(Value::as_str);

        let run_status = match (status, conclusion) {
            ("completed", Some("success")) => TestRunStatus::Passed,
            ("completed", Some("cancelled")) => TestRunStatus::Cancelled,
            ("completed", Some(_)) => TestRunStatus::Failed,
            ("completed", None) => TestRunStatus::Unknown,
            ("queued" | "waiting" | "pending", _) => TestRunStatus::Queued,
            _ => TestRunStatus::Running,
        };

        TestRunOutcome {
            status: run_status,
            provider_run_id: run.get("id").and_then(Value::as_u64),
            run_url: run
                .get("html_url")
                .and_then(Value::as_str)
                .map(str::to_string),
            sha: run
                .get("head_sha")
                .and_then(Value::as_str)
                .map(str::to_string),
            started_at: Self::parse_time(run, "run_started_at"),
            finished_at: match run_status.is_active() {
                true => None,
                false => Self::parse_time(run, "updated_at"),
            },
            totals: None,
            report_state: TestReportState::None,
        }
    }

    fn parse_time(run: &Value, field: &str) -> Option<DateTime<Utc>> {
        let raw = run.get(field).and_then(Value::as_str)?;

        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    }

    /// Артефакт распаковывается выборочно: итоги — в память, HTML-отчёт — на диск.
    /// Пути из архива проверяются, чтобы запись не ушла за пределы каталога
    fn unpack(
        archive: Vec<u8>,
        suite: &TestSuite,
        target_dir: &Path,
        max_size_bytes: u64,
    ) -> Result<(Option<String>, bool), FetchTestRunError> {
        let reader = std::io::Cursor::new(archive);
        let mut zip = zip::ZipArchive::new(reader)
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        let report_prefix = format!("{}/", suite.report_path.trim_end_matches('/'));
        let mut summary = None;
        let mut report_size = 0u64;
        let mut report_too_large = false;

        for index in 0..zip.len() {
            let mut entry = zip
                .by_index(index)
                .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

            let Some(entry_path) = entry.enclosed_name() else {
                continue;
            };
            let entry_path = entry_path.to_path_buf();
            let entry_name = entry_path.to_string_lossy().to_string();

            if entry_name == suite.summary_path {
                let mut content = String::new();

                std::io::Read::read_to_string(&mut entry, &mut content)
                    .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;
                summary = Some(content);

                continue;
            }

            if !entry_name.starts_with(&report_prefix) || entry.is_dir() {
                continue;
            }

            report_size += entry.size();

            if report_size > max_size_bytes {
                report_too_large = true;

                continue;
            }

            let relative = entry_name.trim_start_matches(&report_prefix);
            let destination = target_dir.join(relative);

            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;
            }

            let mut file = std::fs::File::create(&destination)
                .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

            std::io::copy(&mut entry, &mut file)
                .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;
        }

        if report_too_large {
            // Частично распакованный отчёт бесполезен — убираем
            std::fs::remove_dir_all(target_dir).ok();
        }

        Ok((summary, report_too_large))
    }
}

#[async_trait]
impl TestRunner for GithubActionsTestRunner {
    async fn dispatch(
        &self,
        suite: &TestSuite,
        git_ref: &str,
        args: &str,
        tag: &RunTag,
    ) -> Result<(), DispatchTestRunError> {
        let url = self.repository_url(
            suite,
            &format!("actions/workflows/{}/dispatches", suite.workflow_file),
        );
        let body = json!({
            "ref": git_ref,
            "inputs": {
                suite.args_input_name.clone(): args,
                suite.ref_input_name.clone(): git_ref,
                suite.tag_input_name.clone(): tag.as_str(),
            }
        });

        let response = self
            .request(reqwest::Method::POST, url)
            .json(&body)
            .send()
            .await
            .map_err(|error| DispatchTestRunError::ProviderError(error.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(DispatchTestRunError::WorkflowNotFound(
                suite.workflow_file.clone(),
            ));
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            return Err(DispatchTestRunError::ProviderError(format!(
                "{status}: {body}"
            )));
        }

        Ok(())
    }

    async fn find_run_by_tag(
        &self,
        suite: &TestSuite,
        tag: &RunTag,
    ) -> Result<TestRunOutcome, FetchTestRunError> {
        let url = self.repository_url(
            suite,
            &format!(
                "actions/workflows/{}/runs?event=workflow_dispatch&per_page={}",
                suite.workflow_file, RUNS_LOOKUP_LIMIT
            ),
        );

        let response = self
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(FetchTestRunError::ProviderError(
                response.status().to_string(),
            ));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        let runs = body
            .get("workflow_runs")
            .and_then(Value::as_array)
            .ok_or_else(|| FetchTestRunError::ProviderError("Unexpected response".to_string()))?;

        // Метку CI подставляет в имя прогона — по ней прогон и находится
        let run = runs
            .iter()
            .find(|run| {
                run.get("display_title")
                    .or_else(|| run.get("name"))
                    .and_then(Value::as_str)
                    .is_some_and(|title| title.contains(tag.as_str()))
            })
            .ok_or(FetchTestRunError::NotFound)?;

        Ok(Self::build_outcome(run))
    }

    async fn fetch_artifacts(
        &self,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<TestRunArtifacts, FetchTestRunError> {
        let list_url =
            self.repository_url(suite, &format!("actions/runs/{provider_run_id}/artifacts"));

        let response = self
            .request(reqwest::Method::GET, list_url)
            .send()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(FetchTestRunError::ProviderError(
                response.status().to_string(),
            ));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        let artifact = body
            .get("artifacts")
            .and_then(Value::as_array)
            .and_then(|artifacts| {
                artifacts.iter().find(|artifact| {
                    artifact
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name.starts_with(&suite.artifact_prefix))
                })
            })
            .cloned();

        // Прогон мог упасть до тестов — артефакта нет, и это не ошибка
        let Some(artifact) = artifact else {
            return Ok(TestRunArtifacts {
                totals: None,
                failures: Vec::new(),
                report_dir: None,
                report_too_large: false,
            });
        };

        let download_url = artifact
            .get("archive_download_url")
            .and_then(Value::as_str)
            .ok_or_else(|| FetchTestRunError::ProviderError("No download url".to_string()))?;

        let archive = self
            .request(reqwest::Method::GET, download_url.to_string())
            .send()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?
            .bytes()
            .await
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?
            .to_vec();

        let target_dir = self.work_dir.join(provider_run_id.to_string());

        std::fs::create_dir_all(&target_dir)
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        let max_size_bytes = self.report_max_size_mb * BYTES_IN_MEGABYTE;
        let (summary_raw, report_too_large) =
            Self::unpack(archive, suite, &target_dir, max_size_bytes)?;

        let parsed = summary_raw.as_deref().and_then(parse_summary);

        Ok(TestRunArtifacts {
            totals: parsed.as_ref().map(|summary| summary.totals),
            failures: parsed.map(|summary| summary.failures).unwrap_or_default(),
            report_dir: match report_too_large {
                true => None,
                false => Some(target_dir.to_string_lossy().to_string()),
            },
            report_too_large,
        })
    }

    async fn list_blocks(&self, suite: &TestSuite) -> Result<Vec<String>, ListTestBlocksError> {
        let mut blocks = Vec::new();

        for application in self.list_directories(suite, &suite.tests_root).await? {
            let nested_path = format!("{}/{}", suite.tests_root, application);
            let nested = self.list_directories(suite, &nested_path).await?;

            // Блок — каталог второго уровня (accounts/setup); если вложенных нет,
            // блоком считается само приложение
            if nested.is_empty() {
                blocks.push(application.clone());

                continue;
            }

            for block in nested {
                blocks.push(format!("{application}/{block}"));
            }
        }

        blocks.sort();

        Ok(blocks)
    }
}

impl GithubActionsTestRunner {
    async fn list_directories(
        &self,
        suite: &TestSuite,
        path: &str,
    ) -> Result<Vec<String>, ListTestBlocksError> {
        let url = self.repository_url(
            suite,
            &format!("contents/{}?ref={}", path, suite.default_ref),
        );

        let response = self
            .request(reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|error| ListTestBlocksError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(ListTestBlocksError::ProviderError(
                response.status().to_string(),
            ));
        }

        let body: Value = response
            .json()
            .await
            .map_err(|error| ListTestBlocksError::ProviderError(error.to_string()))?;

        let entries = body
            .as_array()
            .ok_or_else(|| ListTestBlocksError::ProviderError("Unexpected response".to_string()))?;

        Ok(entries
            .iter()
            .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("dir"))
            .filter_map(|entry| {
                entry
                    .get("name")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .collect())
    }
}
