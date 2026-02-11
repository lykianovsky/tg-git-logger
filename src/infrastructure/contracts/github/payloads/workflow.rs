use crate::infrastructure::contracts::github::events::GithubEvent;
use crate::utils::builder::message::MessageBuilder;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowEvent {
    pub action: String, // "queued", "in_progress", "completed", "requested", etc.
    pub workflow_job: Option<GithubWorkflowJob>,
    pub workflow_run: Option<GithubWorkflowRun>,
    pub repository: GithubRepository,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowJob {
    pub id: u64,
    pub run_id: u64,
    pub run_url: String,
    pub html_url: String,
    pub status: String,             // "queued", "in_progress", "completed"
    pub conclusion: Option<String>, // "success", "failure", "cancelled" etc.
    pub name: String,
    pub steps: Option<Vec<GithubWorkflowStep>>,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowStep {
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: GithubCommitAuthor,
}

#[derive(Debug, Deserialize)]
pub struct GithubWorkflowRun {
    pub id: u64,
    pub name: String,
    pub html_url: String,
    pub status: String, // "queued", "in_progress", "completed"
    pub conclusion: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub head_commit: Option<GithubCommitInfo>,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub avatar_url: Option<String>,
    pub id: u64,
}

impl GithubWorkflowEvent {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value).map_err(|e| format!("Failed to parse workflow event: {}", e))
    }

    fn format_time(&self, time_str: &str) -> Option<String> {
        DateTime::parse_from_rfc3339(time_str).ok().map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            local.format("%d.%m.%Y %H:%M:%S").to_string()
        })
    }

    fn title(&self) -> String {
        match self.action.as_str() {
            "queued" => "⏳ Workflow поставлен в очередь".to_string(),
            "in_progress" => "🏃 Workflow выполняется".to_string(),
            "completed" => {
                let conclusion = self
                    .workflow_run
                    .as_ref()
                    .and_then(|w| w.conclusion.as_deref())
                    .unwrap_or("unknown");
                match conclusion {
                    "success" => "✅ Workflow успешно завершён".to_string(),
                    "failure" => "❌ Workflow завершён с ошибкой".to_string(),
                    "cancelled" => "⚠️ Workflow отменён".to_string(),
                    _ => format!("ℹ️ Workflow завершён ({})", conclusion),
                }
            }
            _ => format!("ℹ️ Workflow {}", self.action),
        }
    }

    pub fn build(&self) -> MessageBuilder {
        let mut builder = MessageBuilder::new().with_html_escape(true);

        // Заголовок
        builder = builder.bold(&self.title());

        // Время workflow
        if let Some(run) = &self.workflow_run {
            if let Some(time) = self.format_time(&run.updated_at) {
                builder = builder.line(&format!("🕒 <i>{}</i>", time));
            }
        }

        builder = builder.empty_line();

        // Репозиторий
        if let Some(repo_url) = &self.repository.html_url {
            builder = builder.section(
                "📦 Репозиторий",
                &format!("<a href=\"{}\">{}</a>", repo_url, self.repository.full_name),
            );
        } else {
            builder = builder.section("📦 Репозиторий", &self.repository.full_name);
        }

        // Автор события
        builder = builder.section_bold("👤 Инициатор", &self.sender.login);

        // Workflow Job
        if let Some(job) = &self.workflow_job {
            builder = builder.section_code("🏷️ Workflow", &job.name);
            builder = builder.section(
                "🔗 Ссылка",
                &format!("<a href=\"{}\">Перейти</a>", job.html_url),
            );
            builder = builder.section("📌 Статус", &job.status);
            if let Some(conclusion) = &job.conclusion {
                builder = builder.section("✅ Вывод", conclusion);
            }
        } else if let Some(run) = &self.workflow_run {
            builder = builder.section_code("🏷️ Workflow Run", &run.name);
            builder = builder.section(
                "🔗 Ссылка",
                &format!("<a href=\"{}\">Перейти</a>", run.html_url),
            );
            builder = builder.section("📌 Статус", &run.status);
            if let Some(conclusion) = &run.conclusion {
                builder = builder.section("✅ Вывод", &run.conclusion.clone().unwrap_or_default());
            }
        }

        // Commit info
        if let Some(run) = &self.workflow_run {
            if let Some(commit) = &run.head_commit {
                builder = builder.section("📝 Сообщение", commit.message.as_str());
            }
        }

        builder
    }
}

impl GithubEvent for GithubWorkflowEvent {
    fn build(&self) -> MessageBuilder {
        self.build()
    }
}
