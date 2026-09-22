use crate::domain::test_run::entities::test_run::TestRunOutcome;
use crate::domain::test_run::entities::test_suite::TestSuite;
use crate::domain::test_run::ports::test_runner::{
    DispatchTestRunError, FetchTestRunError, ListTestBlocksError, TestRunArtifacts, TestRunner,
};
use crate::domain::test_run::value_objects::run_tag::RunTag;
use crate::domain::test_run::value_objects::test_run_status::TestRunStatus;
use crate::infrastructure::integrations::test_runner::summary::parse_summary;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{Value, json};

/// Сколько последних прогонов просматриваем, разыскивая свой по метке
const RUNS_LOOKUP_LIMIT: u32 = 30;
const GITHUB_API_VERSION: &str = "2022-11-28";
const USER_AGENT: &str = "tg-bot-logger";

pub struct GithubActionsTestRunner {
    http: reqwest::Client,
    api_base: String,
    token: String,
}

impl GithubActionsTestRunner {
    pub fn new(api_base: String, token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base,
            token,
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
        }
    }

    fn parse_time(run: &Value, field: &str) -> Option<DateTime<Utc>> {
        let raw = run.get(field).and_then(Value::as_str)?;

        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|value| value.with_timezone(&Utc))
    }

    /// Из артефакта нужен только машиночитаемый отчёт: HTML-отчёт бот строит сам,
    /// а полный отчёт Playwright остаётся в артефактах прогона
    fn read_summary(
        archive: Vec<u8>,
        suite: &TestSuite,
    ) -> Result<Option<String>, FetchTestRunError> {
        let reader = std::io::Cursor::new(archive);
        let mut zip = zip::ZipArchive::new(reader)
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        let mut entry = match zip.by_name(&suite.summary_path) {
            Ok(entry) => entry,
            Err(_) => return Ok(None),
        };

        let mut content = String::new();

        std::io::Read::read_to_string(&mut entry, &mut content)
            .map_err(|error| FetchTestRunError::ProviderError(error.to_string()))?;

        Ok(Some(content))
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

        let summary_raw = Self::read_summary(archive, suite)?;
        let parsed = summary_raw.as_deref().and_then(parse_summary);

        Ok(TestRunArtifacts {
            totals: parsed.as_ref().map(|summary| summary.totals),
            failures: parsed.map(|summary| summary.failures).unwrap_or_default(),
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
