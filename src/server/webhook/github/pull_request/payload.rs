use crate::server::webhook::github::events::GithubEvent;
use crate::utils::notifier::message_builder::MessageBuilder;
use crate::utils::task_link;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct PullRequestEvent {
    pub action: String, // "opened", "closed", "reopened", "synchronize" и т.д.
    pub number: u64,
    pub pull_request: PullRequest,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub title: String,
    pub html_url: String,
    pub user: User,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
    pub avatar_url: Option<String>,
    pub id: u64,
}

impl PullRequestEvent {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value)
            .map_err(|e| format!("Failed to parse pull request event: {}", e))
    }

    fn format_time(&self, time_str: &str) -> Option<String> {
        DateTime::parse_from_rfc3339(time_str).ok().map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            local.format("%d.%m.%Y %H:%M:%S").to_string()
        })
    }

    fn title(&self) -> String {
        match self.action.as_str() {
            "opened" => "🆕 Pull Request открыт".to_string(),
            "closed" => {
                if self.pull_request.merged_at.is_some() {
                    "🎉 Pull Request смержен".to_string()
                } else {
                    "❌ Pull Request закрыт".to_string()
                }
            }
            "reopened" => "♻️ Pull Request переоткрыт".to_string(),
            "synchronize" => "🔄 Pull Request обновлён".to_string(),
            _ => format!("ℹ️ Pull Request {}", self.action),
        }
    }

    pub fn build(&self) -> MessageBuilder {
        let mut builder = MessageBuilder::new().with_html_escape(true);

        // Заголовок
        builder = builder.bold(&self.title());

        // Время создания PR
        if let Some(created) = self.format_time(&self.pull_request.created_at) {
            builder = builder.line(&format!("🕒 <i>{}</i>", created));
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

        // Автор PR
        builder = builder.section_bold("👤 Автор PR", &self.pull_request.user.login);

        // Заголовок PR
        builder = builder.section(
            "📝 Заголовок PR",
            &task_link::linkify(self.pull_request.title.as_str()),
        );

        // Ссылка на PR
        builder = builder.section(
            "🔗 Ссылка",
            &format!("<a href=\"{}\">Перейти</a>", self.pull_request.html_url),
        );

        builder
    }
}

impl GithubEvent for PullRequestEvent {
    fn build(&self) -> MessageBuilder {
        self.build()
    }
}
