use crate::domain::test_run::entities::test_run::TestRunOutcome;
use crate::domain::test_run::entities::test_suite::TestSuite;
use crate::domain::test_run::ports::test_runner::{
    CiOption, DispatchTestRunError, FetchTestRunError, ListTestBlocksError, TestRunArtifacts,
    TestRunProgress, TestRunner,
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
/// Сколько процессов CI и веток показываем при подключении тестов
const WORKFLOWS_LIMIT: u32 = 50;
const USER_AGENT: &str = "tg-bot-logger";

pub struct GithubActionsTestRunner {
    http: reqwest::Client,
    api_base: String,
}

impl GithubActionsTestRunner {
    pub fn new(api_base: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_base,
        }
    }

    fn request(
        &self,
        token: &str,
        method: reqwest::Method,
        url: String,
    ) -> reqwest::RequestBuilder {
        self.http
            .request(method, url)
            .bearer_auth(token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
    }

    fn url(&self, owner: &str, name: &str, path: &str) -> String {
        format!(
            "{}/repos/{}/{}/{}",
            self.api_base.trim_end_matches('/'),
            owner,
            name,
            path
        )
    }

    /// Неуспешный ответ GitHub — в ошибку, по которой видно, что делать: 404 чинится
    /// настройкой (не та ветка или каталог), 401/403 — доступом, остальное — повтором.
    /// `path` и `git_ref` нужны только 404, поэтому передаются вызывающим
    fn status_error(
        status: reqwest::StatusCode,
        path: Option<(&str, &str)>,
    ) -> ListTestBlocksError {
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return ListTestBlocksError::AccessDenied;
        }

        match (status, path) {
            (reqwest::StatusCode::NOT_FOUND, Some((path, git_ref))) => {
                ListTestBlocksError::PathNotFound {
                    path: path.to_string(),
                    git_ref: git_ref.to_string(),
                }
            }
            _ => ListTestBlocksError::ProviderError(status.to_string()),
        }
    }

    async fn get_json(&self, token: &str, url: String) -> Result<Value, ListTestBlocksError> {
        let response = self
            .request(token, reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|error| ListTestBlocksError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::status_error(response.status(), None));
        }

        response
            .json()
            .await
            .map_err(|error| ListTestBlocksError::ProviderError(error.to_string()))
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

        let run_status = TestRunStatus::from_provider(status, conclusion);

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

        // В архиве путь может быть коротким: upload-artifact кладёт содержимое папки в корень
        let file_name = suite
            .summary_path
            .rsplit('/')
            .next()
            .unwrap_or(&suite.summary_path)
            .to_string();
        let path_in_archive = (0..zip.len())
            .filter_map(|index| {
                zip.by_index(index)
                    .ok()
                    .and_then(|entry| entry.enclosed_name())
            })
            .map(|path| path.to_string_lossy().to_string())
            .find(|path| {
                path == &suite.summary_path
                    || path.ends_with(&format!("/{file_name}"))
                    || path == &file_name
            });

        let Some(path_in_archive) = path_in_archive else {
            tracing::warn!(
                summary_path = %suite.summary_path,
                "Summary file is missing in artifact"
            );

            return Ok(None);
        };

        let mut entry = match zip.by_name(&path_in_archive) {
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
    /// Список процессов CI репозитория. Запустить через `workflow_dispatch` можно только
    /// тот, чей файл есть в ветке по умолчанию, — это ограничение GitHub
    async fn list_workflows(
        &self,
        token: &str,
        owner: &str,
        name: &str,
    ) -> Result<Vec<CiOption>, ListTestBlocksError> {
        let body = self
            .get_json(
                token,
                self.url(
                    owner,
                    name,
                    &format!("actions/workflows?per_page={WORKFLOWS_LIMIT}"),
                ),
            )
            .await?;

        let workflows = body
            .get("workflows")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        tracing::debug!(
            owner = %owner,
            repository = %name,
            count = workflows.len(),
            "GitHub workflows fetched"
        );

        Ok(workflows
            .iter()
            .filter_map(|workflow| {
                // В списке нужен файл: именно его принимает запуск workflow_dispatch
                let path = workflow.get("path").and_then(Value::as_str)?;
                let file = path.rsplit('/').next()?.to_string();
                let label = workflow
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or(&file)
                    .to_string();

                Some(CiOption { value: file, label })
            })
            .collect())
    }

    async fn dispatch(
        &self,
        token: &str,
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
            .request(token, reqwest::Method::POST, url)
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
        token: &str,
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
            .request(token, reqwest::Method::GET, url)
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
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<TestRunArtifacts, FetchTestRunError> {
        let list_url =
            self.repository_url(suite, &format!("actions/runs/{provider_run_id}/artifacts"));

        let response = self
            .request(token, reqwest::Method::GET, list_url)
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
            .request(token, reqwest::Method::GET, download_url.to_string())
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

    async fn fetch_progress(
        &self,
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<Option<TestRunProgress>, FetchTestRunError> {
        let url = self.repository_url(suite, &format!("actions/runs/{provider_run_id}/jobs"));

        let response = self
            .request(token, reqwest::Method::GET, url)
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

        // Прогон тестов — одна задача; если задач несколько, берём ту, что идёт
        let job = body
            .get("jobs")
            .and_then(Value::as_array)
            .and_then(|jobs| {
                jobs.iter()
                    .find(|job| job.get("status").and_then(Value::as_str) == Some("in_progress"))
                    .or_else(|| jobs.first())
            })
            .cloned();

        let Some(job) = job else {
            return Ok(None);
        };

        let steps = job
            .get("steps")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if steps.is_empty() {
            return Ok(None);
        }

        let completed_steps = steps
            .iter()
            .filter(|step| step.get("status").and_then(Value::as_str) == Some("completed"))
            .count() as u32;
        let current_step = steps
            .iter()
            .find(|step| step.get("status").and_then(Value::as_str) == Some("in_progress"))
            .and_then(|step| step.get("name").and_then(Value::as_str))
            .map(str::to_string);

        Ok(Some(TestRunProgress {
            completed_steps,
            total_steps: steps.len() as u32,
            current_step,
        }))
    }

    async fn cancel_run(
        &self,
        token: &str,
        suite: &TestSuite,
        provider_run_id: u64,
    ) -> Result<(), DispatchTestRunError> {
        let url = self.repository_url(suite, &format!("actions/runs/{provider_run_id}/cancel"));

        let response = self
            .request(token, reqwest::Method::POST, url)
            .send()
            .await
            .map_err(|error| DispatchTestRunError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(DispatchTestRunError::ProviderError(
                response.status().to_string(),
            ));
        }

        Ok(())
    }

    async fn list_blocks(
        &self,
        token: &str,
        suite: &TestSuite,
    ) -> Result<Vec<String>, ListTestBlocksError> {
        let mut blocks = Vec::new();

        for application in self
            .list_directories(token, suite, &suite.tests_root)
            .await?
        {
            let nested_path = format!("{}/{}", suite.tests_root, application);
            let nested = self.list_directories(token, suite, &nested_path).await?;

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
        token: &str,
        suite: &TestSuite,
        path: &str,
    ) -> Result<Vec<String>, ListTestBlocksError> {
        let url = self.repository_url(
            suite,
            &format!("contents/{}?ref={}", path, suite.default_ref),
        );

        let response = self
            .request(token, reqwest::Method::GET, url)
            .send()
            .await
            .map_err(|error| ListTestBlocksError::ProviderError(error.to_string()))?;

        if !response.status().is_success() {
            return Err(Self::status_error(
                response.status(),
                Some((path, &suite.default_ref)),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_names_directory_and_branch() {
        let error = GithubActionsTestRunner::status_error(
            reqwest::StatusCode::NOT_FOUND,
            Some(("e2e/tests", "dev")),
        );

        assert!(matches!(
            error,
            ListTestBlocksError::PathNotFound { path, git_ref }
                if path == "e2e/tests" && git_ref == "dev"
        ));
    }

    #[test]
    fn missing_path_without_context_stays_provider_error() {
        let error = GithubActionsTestRunner::status_error(reqwest::StatusCode::NOT_FOUND, None);

        assert!(matches!(error, ListTestBlocksError::ProviderError(_)));
    }

    #[test]
    fn unauthorized_and_forbidden_are_access_denied() {
        for status in [
            reqwest::StatusCode::UNAUTHORIZED,
            reqwest::StatusCode::FORBIDDEN,
        ] {
            let error = GithubActionsTestRunner::status_error(status, Some(("e2e/tests", "dev")));

            assert!(matches!(error, ListTestBlocksError::AccessDenied));
        }
    }

    #[test]
    fn server_error_keeps_status_for_retry() {
        let error = GithubActionsTestRunner::status_error(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            Some(("e2e/tests", "dev")),
        );

        assert!(matches!(error, ListTestBlocksError::ProviderError(_)));
    }
}
