use crate::server::webhook::github::events::GithubEvent;
use crate::utils::notifier::message_builder::MessageBuilder;
use chrono::{DateTime, Local};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct ReleaseEvent {
    pub action: String, // "published", "created", "edited", "deleted", "prereleased"
    pub release: Release,
    pub repository: Repository,
    pub sender: User,
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub id: u64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub html_url: String,
    pub author: User,
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

impl ReleaseEvent {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value).map_err(|e| format!("Failed to parse release event: {}", e))
    }

    fn format_time(&self, time_str: &str) -> Option<String> {
        DateTime::parse_from_rfc3339(time_str).ok().map(|dt| {
            let local: DateTime<Local> = dt.with_timezone(&Local);
            local.format("%d.%m.%Y %H:%M:%S").to_string()
        })
    }

    fn title(&self) -> String {
        match self.action.as_str() {
            "published" => "🚀 Релиз опубликован".to_string(),
            "created" => {
                if self.release.draft {
                    "📝 Черновик релиза создан".to_string()
                } else if self.release.prerelease {
                    "⚡ Pre-release создан".to_string()
                } else {
                    "🆕 Релиз создан".to_string()
                }
            }
            "edited" => "✏️ Релиз отредактирован".to_string(),
            "deleted" => "🗑️ Релиз удалён".to_string(),
            "prereleased" => "⚡ Pre-release опубликован".to_string(),
            _ => format!("ℹ️ Релиз {}", self.action),
        }
    }

    pub fn build(&self) -> MessageBuilder {
        let mut builder = MessageBuilder::new().with_html_escape(true);

        // Заголовок
        builder = builder.bold(&self.title());

        // Время
        if let Some(time) = self
            .release
            .published_at
            .as_deref()
            .or(Some(&self.release.created_at))
            .and_then(|t| self.format_time(t))
        {
            builder = builder.line(&format!("🕒 <i>{}</i>", time));
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

        // Автор релиза
        builder = builder.section_bold("👤 Автор", &self.release.author.login);

        // Тэг и название релиза
        builder = builder.section_code("🏷️ Тэг", &self.release.tag_name);
        if let Some(name) = &self.release.name {
            builder = builder.section("📌 Название", name);
        }

        // Ссылка на релиз
        builder = builder.section(
            "🔗 Ссылка",
            &format!("<a href=\"{}\">Перейти</a>", self.release.html_url),
        );

        builder
    }
}

impl GithubEvent for ReleaseEvent {
    fn build(&self) -> MessageBuilder {
        self.build()
    }
}
