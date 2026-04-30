use crate::domain::shared::events::event::DomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookWorkflowEvent {
    pub id: u64,                    // уникальный ID workflow run
    pub name: String,               // имя workflow
    pub run_number: u64,            // номер запуска
    pub head_branch: String,        // ветка
    pub head_sha: String,           // коммит
    pub status: String,             // queued, in_progress, completed
    pub conclusion: Option<String>, // success, failure, cancelled
    pub html_url: Option<String>,   // ссылка на GitHub

    pub actor: Option<String>, // кто инициировал workflow

    pub repo: String,             // полное имя репозитория
    pub repo_url: Option<String>, // ссылка на репозиторий

    pub created_at: Option<String>, // дата создания
    pub updated_at: Option<String>, // дата последнего обновления
}

impl WebhookEvent for WebhookWorkflowEvent {
    fn build_text(&self) -> String {
        let title = match self.conclusion.as_deref() {
            Some("success") => "✅ Workflow завершён успешно",
            Some("failure") => "❌ Workflow завершился с ошибкой",
            Some("cancelled") => "🚫 Workflow отменён",
            Some("skipped") => "⏭️ Workflow пропущен",
            Some("timed_out") => "⏱️ Workflow превысил время ожидания",
            None => match self.status.as_str() {
                "queued" => "⏳ Workflow в очереди",
                "in_progress" => "🔄 Workflow выполняется",
                _ => "🔧 Workflow",
            },
            _ => "🔧 Workflow",
        };

        let status_label = match self.status.as_str() {
            "queued" => "⏳ В очереди",
            "in_progress" => "🔄 Выполняется",
            "completed" => "✅ Завершён",
            other => other,
        };

        let short_sha = &self.head_sha[..7.min(self.head_sha.len())];
        let safe_repo = MessageBuilder::escape_html(&self.repo);

        // ── Заголовок ──────────────────────────────────────
        let mut builder = MessageBuilder::new().bold(title).empty_line();

        // ── Основная инфо ──────────────────────────────────
        builder = builder
            .section_bold("⚙️ Workflow", &MessageBuilder::escape_html(&self.name))
            .section("🔢 Запуск", &format!("<b>#{}</b>", self.run_number));

        if let Some(actor) = &self.actor {
            builder = builder.section_bold("👤 Инициатор", &MessageBuilder::escape_html(actor));
        }

        builder = builder.empty_line();

        // ── Коммит и ветка ─────────────────────────────────
        builder = builder
            .section_code("🌿 Ветка", &MessageBuilder::escape_html(&self.head_branch))
            .section_code("🔐 Коммит", &MessageBuilder::escape_html(short_sha))
            .empty_line();

        // ── Статус ─────────────────────────────────────────
        builder = builder.section("📌 Статус", status_label);

        if let Some(conclusion) = &self.conclusion {
            let conclusion_label = match conclusion.as_str() {
                "success" => "✅ Успешно",
                "failure" => "❌ Ошибка",
                "cancelled" => "🚫 Отменён",
                "skipped" => "⏭️ Пропущен",
                "timed_out" => "⏱️ Таймаут",
                other => other,
            };
            builder = builder.section("🏁 Результат", conclusion_label);
        }

        builder = builder.empty_line();

        // ── Временные метки ────────────────────────────────
        if let Some(created) = &self.created_at {
            builder = builder.section("🕒 Запущен", created);
        }

        if let Some(updated) = &self.updated_at {
            builder = builder.section("🔄 Обновлён", updated);
        }

        builder = builder.empty_line();

        // ── Ссылки ─────────────────────────────────────────
        if let Some(url) = &self.html_url {
            let trimmed = url.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                builder = builder.section(
                    "🔗 Workflow",
                    &format!(
                        "<a href=\"{}\">Просмотреть →</a>",
                        MessageBuilder::escape_html(trimmed)
                    ),
                );
            }
        }

        match &self.repo_url {
            Some(url) if url.trim().starts_with("http://") || url.trim().starts_with("https://") => {
                builder = builder.section(
                    "📦 Репозиторий",
                    &format!(
                        "<a href=\"{}\">{}</a>",
                        MessageBuilder::escape_html(url.trim()),
                        safe_repo
                    ),
                )
            }
            _ => builder = builder.section("📦 Репозиторий", &safe_repo),
        }

        builder.build()
    }
}

impl DomainEvent for WebhookWorkflowEvent {
    const EVENT_NAME: &'static str = "webhook.workflow";
}

impl MessageBrokerMessage for WebhookWorkflowEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
