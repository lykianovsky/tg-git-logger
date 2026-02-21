use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPullRequestEvent {
    pub source: String, // кто вызвал событие (lykianovsky)
    pub author: String, // автор PR (schwarz0ker)
    pub repo: String,
    pub repo_url: Option<String>,
    pub title: String, // заголовок PR
    pub number: u64,   // номер PR
    pub action: String,
    pub merged: bool,
    pub merged_by: Option<String>, // кто смержил
    pub draft: bool,
    pub state: String,                // open / closed / merged
    pub head_ref: String,             // ветка источника
    pub base_ref: String,             // ветка назначения
    pub head_repo: String,            // owner:branch источника
    pub base_repo: String,            // owner:branch назначения
    pub pr_url: Option<String>,       // ссылка на PR
    pub merge_commit: Option<String>, // хэш merge commit
    pub assignees: Vec<String>,       // назначенные
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub commits: u64, // кол-во коммитов
    pub additions: u64,
    pub deletions: u64,
    pub changed_files: u64,
}

impl WebhookEvent for WebhookPullRequestEvent {
    fn build_text(&self) -> String {
        let title = match self.action.as_str() {
            "opened" => "🔀 Открыт Pull Request",
            "closed" if self.merged => "🎉 Pull Request смержен",
            "closed" => "❌ Pull Request закрыт",
            "synchronize" => "🔄 Pull Request обновлён",
            "reopened" => "🔁 Pull Request переоткрыт",
            _ => "🔀 Pull Request",
        };

        let state_label = if self.merged {
            "🎉 Смёржен"
        } else if self.state == "closed" {
            "❌ Закрыт"
        } else {
            "🟢 Открыт"
        };

        let short_merge_commit = self
            .merge_commit
            .as_deref()
            .map(|h| &h[..7.min(h.len())])
            .unwrap_or("-");

        let assignees = if self.assignees.is_empty() {
            "—".to_string()
        } else {
            self.assignees.join(", ")
        };

        // ── Заголовок ──────────────────────────────────────
        let mut builder = MessageBuilder::new()
            .bold(&format!("{} #{}", title, self.number))
            .empty_line();

        // ── PR инфо ────────────────────────────────────────
        builder = builder
            .section_bold("👤 Автор", &self.author)
            .section("📝 Заголовок", &self.title)
            .empty_line();

        // ── Временные метки ────────────────────────────────
        builder = builder
            .section("🕒 Создан", &self.created_at)
            .section("🔄 Обновлён", &self.updated_at);

        if let Some(merged_at) = &self.merged_at {
            builder = builder.section("🎉 Смёржен", merged_at);
        }

        builder = builder.empty_line();

        // ── Ветки ──────────────────────────────────────────
        builder = builder
            .section(
                "🔀 Ветки",
                &format!(
                    "<code>{}</code> → <code>{}</code>",
                    self.head_repo, self.base_repo
                ),
            )
            .empty_line();

        // ── Статус ─────────────────────────────────────────
        builder = builder.section("📌 Состояние", state_label);

        if let Some(merged_by) = &self.merged_by {
            builder = builder.section_bold("🎉 Смёржил", merged_by);
        }

        if let Some(commit) = &self.merge_commit {
            let short = &commit[..7.min(commit.len())];
            builder = builder.section("🔐 Merge commit", &format!("<code>{}</code>", short));
        }

        builder = builder
            .section("👥 Назначены", &assignees)
            .section_bold("⚡️ Инициатор", &self.source)
            .empty_line();

        // ── Статистика ─────────────────────────────────────
        builder = builder
            .bold("📊 Изменения")
            .line(&format!("├ 🔢 Коммитов: <b>{}</b>", self.commits))
            .line(&format!("├ ➕ Добавлено строк: <b>{}</b>", self.additions))
            .line(&format!("├ ➖ Удалено строк: <b>{}</b>", self.deletions))
            .line(&format!(
                "└ 📂 Файлов изменено: <b>{}</b>",
                self.changed_files
            ))
            .empty_line();

        // ── Ссылки ─────────────────────────────────────────
        if let Some(url) = &self.pr_url {
            builder = builder.section(
                "🔗 Pull Request",
                &format!("<a href=\"{}\">Перейти →</a>", url),
            );
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

impl StaticDomainEvent for WebhookPullRequestEvent {
    const EVENT_NAME: &'static str = "webhook.pull_request";
}
