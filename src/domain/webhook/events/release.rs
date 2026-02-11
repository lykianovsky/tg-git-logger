use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookReleaseEvent {
    pub id: u64,                      // уникальный ID релиза
    pub tag_name: String,             // тег релиза
    pub target_commitish: String,     // на какой ветке/коммите основан релиз
    pub name: Option<String>,         // название релиза
    pub body: Option<String>,         // описание релиза
    pub draft: bool,                  // черновик
    pub prerelease: bool,             // предварительный релиз
    pub created_at: Option<String>,   // дата создания
    pub published_at: Option<String>, // дата публикации
    pub html_url: Option<String>,     // ссылка на релиз
    pub author: Option<String>,       // кто создал релиз

    pub repo: String,             // полный идентификатор репозитория
    pub repo_url: Option<String>, // ссылка на репозиторий
}

impl WebhookEvent for WebhookReleaseEvent {
    fn build_text(&self) -> String {
        let title = if self.draft {
            "📝 Черновик релиза"
        } else if self.prerelease {
            "🧪 Предварительный релиз"
        } else {
            "🎉 Новый релиз"
        };

        // ── Заголовок ──────────────────────────────────────
        let mut builder = MessageBuilder::new().bold(title).empty_line();

        // ── Основная инфо ──────────────────────────────────
        if let Some(name) = &self.name {
            builder = builder.section_bold("📌 Название", name);
        }

        builder = builder.section_code("🏷️ Тег", &self.tag_name);
        builder = builder.section_code("🌿 Ветка", &self.target_commitish);

        if let Some(author) = &self.author {
            builder = builder.section_bold("👤 Автор", author);
        }

        builder = builder.empty_line();

        // ── Временные метки ────────────────────────────────
        if let Some(created) = &self.created_at {
            builder = builder.section("🕒 Создан", created);
        }

        if let Some(published) = &self.published_at {
            builder = builder.section("📅 Опубликован", published);
        }

        builder = builder.empty_line();

        // ── Флаги ──────────────────────────────────────────
        let mut flags = Vec::new();
        if self.draft {
            flags.push("📝 Черновик");
        }
        if self.prerelease {
            flags.push("🧪 Pre-release");
        }
        if !flags.is_empty() {
            builder = builder.section("⚑ Тип", &flags.join(" · "));
            builder = builder.empty_line();
        }

        // ── Описание ───────────────────────────────────────
        if let Some(body) = &self.body {
            if !body.trim().is_empty() {
                let truncated = if body.len() > 500 {
                    format!("{}…", &body[..500])
                } else {
                    body.clone()
                };

                builder = builder.bold("📋 Описание").line(&truncated).empty_line();
            }
        }

        // ── Ссылки ─────────────────────────────────────────
        if let Some(url) = &self.html_url {
            builder = builder.section("🔗 Релиз", &format!("<a href=\"{}\">Перейти →</a>", url));
        }

        match &self.repo_url {
            Some(url) => {
                builder = builder.section(
                    "📦 Репозиторий",
                    &format!("<a href=\"{}\">{}</a>", url, self.repo),
                )
            }
            None => builder = builder.section("📦 Репозиторий", &self.repo),
        }

        builder.build()
    }
}

impl StaticDomainEvent for WebhookReleaseEvent {
    const EVENT_NAME: &'static str = "webhook.release";
}
