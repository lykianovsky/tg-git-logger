use crate::infrastructure::contracts::github::events::GithubEvent;
use crate::utils::builder::message::MessageBuilder;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubPullRequestEvent {
    pub action: String, // opened, closed, reopened, synchronize и т.д.
    pub number: u64,
    pub pull_request: GithubPullRequest,
    pub repository: GithubRepository,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubPullRequest {
    pub title: String,
    pub body: Option<String>,
    pub html_url: String,

    pub state: String,
    pub draft: bool,

    pub user: GithubUser,
    pub assignee: Option<GithubUser>,
    pub assignees: Vec<GithubUser>,

    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub merged_at: Option<String>,

    pub merge_commit_sha: Option<String>,
    pub merged: bool,
    pub merged_by: Option<GithubUser>,

    pub commits: u64,
    pub additions: u64,
    pub deletions: u64,
    pub changed_files: u64,

    pub base: GithubPullRequestBranch,
    pub head: GithubPullRequestBranch,
}

#[derive(Debug, Deserialize)]
pub struct GithubPullRequestBranch {
    pub label: String, // user:branch
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    pub repo: GithubRepository,
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

impl GithubPullRequestEvent {
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
                if self.pull_request.merged {
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

    fn human_state(&self) -> &'static str {
        match self.pull_request.state.as_str() {
            "open" => "🟢 Открыт",
            "closed" if self.pull_request.merged => "🎉 Смёржен",
            "closed" => "🔴 Закрыт",
            _ => "❔ Неизвестно",
        }
    }

    pub fn build(&self) -> MessageBuilder {
        let mut builder = MessageBuilder::new().with_html_escape(true);

        // ===== Заголовок =====
        let title = format!("{} #{}", self.title(), self.number);
        builder = builder.bold(&title);

        // ===== Draft =====
        if self.pull_request.draft {
            builder = builder.line("📝 <i>Draft Pull Request</i>");
        }

        // ===== Автор =====
        builder = builder.section_bold("👤 Автор PR", &self.pull_request.user.login);

        builder = builder.empty_line();

        // ===== Заголовок PR =====
        builder = builder.section(
            "📝 Заголовок PR",
            self.pull_request.title.as_str(),
        );

        builder = builder.empty_line();

        // ===== Тайминги =====
        if let Some(created) = self.format_time(&self.pull_request.created_at) {
            builder = builder.line(&format!("🕒 <i>Создан: {}</i>", created));
        }

        if let Some(updated) = self.format_time(&self.pull_request.updated_at) {
            builder = builder.line(&format!("🔄 <i>Обновлён: {}</i>", updated));
        }

        if let Some(merged) = &self.pull_request.merged_at {
            if let Some(time) = self.format_time(merged) {
                builder = builder.line(&format!("🎉 <i>Смёржен: {}</i>", time));
            }
        }

        builder = builder.empty_line();

        // ===== Ветки =====
        builder = builder.section(
            "🔀 Ветки",
            &format!(
                "<code>{}</code> → <code>{}</code>",
                self.pull_request.head.label, self.pull_request.base.label
            ),
        );

        if self.pull_request.head.repo.full_name != self.pull_request.base.repo.full_name {
            builder = builder.line("⚠️ Pull Request из форка");
        }

        builder = builder.empty_line();

        // ===== Состояние =====
        builder = builder.section("📌 Состояние", self.human_state());

        // ===== Кто смержил =====
        if let Some(user) = &self.pull_request.merged_by {
            builder = builder.section("🎉 Смёржил", &user.login);
        }

        // ===== Merge commit =====
        if let Some(sha) = &self.pull_request.merge_commit_sha {
            builder = builder.section("🔐 Merge commit", &format!("<code>{}</code>", &sha[..7]));
        }

        // ===== Ассайны =====
        if !self.pull_request.assignees.is_empty() {
            let users = self
                .pull_request
                .assignees
                .iter()
                .map(|u| u.login.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            builder = builder.section("👥 Назначены", &users);
        }

        // ===== Кто вызвал событие =====
        builder = builder.section("⚡ Событие вызвал", &self.sender.login);

        builder = builder.empty_line();

        // ===== Статистика =====
        builder = builder.section(
            "📊 Изменения",
            &format!(
                "Коммитов: <b>{}</b>\n➕ Добавлено: <b>{}</b>\n➖ Удалено: <b>{}</b>\n📂 Файлов: <b>{}</b>",
                self.pull_request.commits,
                self.pull_request.additions,
                self.pull_request.deletions,
                self.pull_request.changed_files
            ),
        );

        builder = builder.empty_line();

        // ===== Ссылка =====
        builder = builder.section(
            "🔗 Pull Request",
            &format!("<a href=\"{}\">Перейти</a>", self.pull_request.html_url),
        );

        builder = builder.empty_line();

        // ===== Репозиторий =====
        if let Some(repo_url) = &self.repository.html_url {
            builder = builder.section(
                "📦 Репозиторий",
                &format!("<a href=\"{}\">{}</a>", repo_url, self.repository.full_name),
            );
        } else {
            builder = builder.section("📦 Репозиторий", &self.repository.full_name);
        }

        builder
    }
}

impl GithubEvent for GithubPullRequestEvent {
    fn build(&self) -> MessageBuilder {
        self.build()
    }
}