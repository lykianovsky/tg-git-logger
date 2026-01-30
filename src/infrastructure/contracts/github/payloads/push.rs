use crate::infrastructure::contracts::github::events::GithubEvent;
use crate::utils::builder::message::MessageBuilder;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubPushEvent {
    #[serde(rename = "ref")]
    pub ref_field: String, // "refs/heads/main"
    pub before: String,
    pub after: String,
    pub compare: Option<String>, // ссылка на сравнение, может быть пустой
    pub created: Option<bool>,   // иногда отсутствует
    pub deleted: Option<bool>,
    pub forced: Option<bool>,

    pub head_commit: Option<GithubCommit>,

    pub commits: Vec<GithubCommit>,

    pub repository: GithubRepository,
    pub pusher: GithubPusher,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepository {
    pub id: u64,
    pub node_id: Option<String>,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubPusher {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: u64,
    pub node_id: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommit {
    pub id: String,
    pub tree_id: Option<String>,
    pub distinct: Option<bool>,
    pub message: String,
    pub timestamp: Option<String>,
    pub url: String, // required
    pub added: Option<Vec<String>>,
    pub removed: Option<Vec<String>>,
    pub modified: Option<Vec<String>>,

    pub author: Option<CommitAuthor>,
    pub committer: Option<CommitAuthor>,
}

#[derive(Debug, Deserialize)]
pub struct CommitAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl GithubPushEvent {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value).map_err(|e| format!("Failed to parse push event: {}", e))
    }

    fn format_commit_time(&self) -> Option<String> {
        let ts = self.head_commit.as_ref()?.timestamp.as_ref()?;

        return DateTime::parse_from_rfc3339(ts).ok().map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            local.format("%d.%m.%Y %H:%M:%S").to_string()
        });
    }

    fn title(&self) -> &'static str {
        if self.deleted.unwrap_or(false) {
            return "🗑️ Ветка удалена";
        }

        if self.created.unwrap_or(false) {
            return "🌱 Новая ветка создана";
        }

        if self.forced.unwrap_or(false) {
            return "⚠️ Принудительные изменения";
        }

        return "🚀 Новые изменения";
    }

    fn build(&self) -> MessageBuilder {
        let branch = self
            .ref_field
            .strip_prefix("refs/heads/")
            .unwrap_or(&self.ref_field);

        let commits_count = self.commits.len();

        let mut builder = MessageBuilder::new().with_html_escape(true);

        builder = builder.bold(self.title());

        // ===== Время =====
        if let Some(time) = self.format_commit_time() {
            builder = builder.line(&format!("🕒 <i>{}</i>", time));
        }

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

        // ===== Ветка =====
        builder = builder.section_code("🌿 Ветка", branch);

        // ===== Кто пушнул =====
        builder = builder.section_bold("👤 Автор", &self.pusher.name);

        // ===== Коммиты =====
        builder = builder
            .section("🔢 Кол-во коммитов", &commits_count.to_string())
            .empty_line();

        // ===== Список коммитов =====
        if !self.commits.is_empty() {
            builder = builder.bold("📝 Коммиты:");

            let max_commits = 5;

            for commit in self.commits.iter().take(max_commits) {
                let short_hash = &commit.id[..7.min(commit.id.len())];
                let author = commit
                    .author
                    .as_ref()
                    .and_then(|a| a.name.as_deref())
                    .unwrap_or("unknown");

                let raw_message = commit.message.lines().next().unwrap_or("");

                builder = builder.line(&format!(
                    "• <code>{}</code> — {} <i>({})</i>",
                    short_hash, raw_message, author
                ));
            }

            if commits_count > max_commits {
                builder =
                    builder.line(&format!("… и ещё {} коммитов", commits_count - max_commits));
            }

            builder = builder.empty_line();
        }

        // ===== Compare =====
        if let Some(compare_url) = &self.compare {
            builder = builder.section(
                "🔗 Compare",
                &format!("<a href=\"{}\">Просмотреть изменения</a>", compare_url),
            );
        }

        builder
    }
}

impl GithubEvent for GithubPushEvent {
    fn build(&self) -> MessageBuilder {
        self.build()
    }
}
