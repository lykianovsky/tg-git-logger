use crate::domain::shared::events::event::DomainEvent;
use crate::domain::webhook::entities::commit::WebhookCommit;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPushEvent {
    pub source: String,           // кто пушнул
    pub repo: String,             // полное имя репозитория
    pub repo_url: Option<String>, // ссылка на репозиторий
    #[serde(rename = "ref")]
    pub ref_field: String, // ветка, например "refs/heads/main"

    pub before: String,              // старый коммит
    pub after: String,               // новый коммит
    pub compare_url: Option<String>, // ссылка на GitHub Compare
    pub created: bool,               // создана ли ветка
    pub deleted: bool,               // удалена ли ветка
    pub forced: bool,                // форс-пуш

    pub commits: Vec<WebhookCommit>, // список коммитов
}

impl WebhookEvent for WebhookPushEvent {
    fn build_text(&self) -> String {
        let branch = self
            .ref_field
            .strip_prefix("refs/heads/")
            .unwrap_or(&self.ref_field);

        let title = if self.deleted {
            "🗑️ Ветка удалена"
        } else if self.created {
            "🌱 Ветка создана"
        } else if self.forced {
            "⚠️ Принудительный пуш"
        } else {
            "🚀 Новые изменения"
        };

        let short_before = &self.before[..7.min(self.before.len())];
        let short_after = &self.after[..7.min(self.after.len())];

        // ── Заголовок ──────────────────────────────────────
        let mut builder = MessageBuilder::new().bold(title).empty_line();

        // ── Основная инфо ──────────────────────────────────
        builder = builder.section_bold("👤 Автор", &self.source);

        match &self.repo_url {
            Some(url) => {
                builder = builder.section(
                    "📦 Репозиторий",
                    &format!("<a href=\"{}\">{}</a>", url, self.repo),
                )
            }
            None => builder = builder.section("📦 Репозиторий", &self.repo),
        }

        builder = builder.section_code("🌿 Ветка", branch).empty_line();

        // ── Коммиты ────────────────────────────────────────
        builder = builder.section("🔢 Коммитов", &format!("<b>{}</b>", self.commits.len()));

        builder = builder.section(
            "🔀 Изменения",
            &format!(
                "<code>{}</code> → <code>{}</code>",
                short_before, short_after
            ),
        );

        builder = builder.empty_line();

        // ── Список коммитов ────────────────────────────────
        if !self.commits.is_empty() {
            builder = builder.bold("📝 Коммиты");

            let max = 5;
            for commit in self.commits.iter().take(max) {
                let short_hash = &commit.id[..7.min(commit.id.len())];
                let author = commit.author.as_str();
                let message = commit.message.lines().next().unwrap_or("");

                builder = builder.line(&format!(
                    "├ <code>{}</code> <i>({})</i>\n│   {}",
                    short_hash, author, message
                ));
            }

            if self.commits.len() > max {
                builder = builder.line(&format!(
                    "└ <i>… и ещё {} коммитов</i>",
                    self.commits.len() - max
                ));
            }

            builder = builder.empty_line();
        }

        // ── Ссылки ─────────────────────────────────────────
        if let Some(url) = &self.compare_url {
            builder = builder.section(
                "🔗 Compare",
                &format!("<a href=\"{}\">Просмотреть изменения →</a>", url),
            );
        }

        builder.build()
    }
}

impl DomainEvent for WebhookPushEvent {
    const EVENT_NAME: &'static str = "webhook.push";
}

impl MessageBrokerMessage for WebhookPushEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
