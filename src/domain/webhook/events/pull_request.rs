use crate::domain::shared::events::event::DomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString};

#[derive(Debug, PartialEq, Serialize, Deserialize, EnumString, AsRefStr)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WebhookPullRequestEventActionType {
    Opened,      // PR создан
    Closed,      // PR закрыт (merged или просто closed — смотри pr.merged)
    Reopened,    // PR переоткрыт
    Edited,      // изменили title/body/base branch
    Synchronize, // новый коммит запушен в ветку PR

    // Ревью реквесты
    ReviewRequested,      // попросили кого-то зарепьюить
    ReviewRequestRemoved, // убрали реквест на ревью

    // Assignee
    Assigned,   // назначили assignee
    Unassigned, // убрали assignee

    // Лейблы
    Labeled,   // добавили лейбл
    Unlabeled, // убрали лейбл

    // Draft
    ConvertedToDraft, // перевели в драфт
    ReadyForReview,   // вывели из драфта

    // Locked/Unlocked
    Locked,
    Unlocked,
    Milestoned,
    Demilestoned,
    AutoMergeEnabled,
    AutoMergeDisabled,

    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPullRequestEvent {
    pub source: String, // кто вызвал событие (lykianovsky)
    pub author: String, // автор PR (schwarz0ker)
    pub repo: String,
    pub repo_url: Option<String>,
    pub title: String, // заголовок PR
    #[serde(default)]
    pub body: Option<String>, // тело PR
    pub number: u64,   // номер PR
    pub action: WebhookPullRequestEventActionType,
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
    #[serde(default)]
    pub requested_reviewer: Option<String>, // login назначенного на ревью (только для action=ReviewRequested)
    #[serde(default)]
    pub requested_reviewers: Vec<String>, // все ожидающие ревьюеры на момент события
    #[serde(default)]
    pub mergeable_state: Option<String>, // GitHub: clean / dirty / unknown / blocked / behind / draft
}

impl WebhookEvent for WebhookPullRequestEvent {
    fn build_text(&self) -> String {
        let title = match self.action.as_ref() {
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

        let assignees_text = if self.assignees.is_empty() {
            "—".to_string()
        } else {
            self.assignees
                .iter()
                .map(|a| MessageBuilder::escape_html(a))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let safe_author = MessageBuilder::escape_html(&self.author);
        let safe_title = MessageBuilder::escape_html(&self.title);
        let safe_head_repo = MessageBuilder::escape_html(&self.head_repo);
        let safe_base_repo = MessageBuilder::escape_html(&self.base_repo);
        let safe_source = MessageBuilder::escape_html(&self.source);
        let safe_repo = MessageBuilder::escape_html(&self.repo);

        // ── Заголовок ──────────────────────────────────────
        let mut builder = MessageBuilder::new()
            .bold(&format!("{} #{}", title, self.number))
            .empty_line();

        // ── PR инфо ────────────────────────────────────────
        builder = builder
            .section_bold("👤 Автор", &safe_author)
            .section("📝 Заголовок", &safe_title)
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
                    safe_head_repo, safe_base_repo
                ),
            )
            .empty_line();

        // ── Статус ─────────────────────────────────────────
        builder = builder.section("📌 Состояние", state_label);

        if let Some(merged_by) = &self.merged_by {
            builder = builder.section_bold("🎉 Смёржил", &MessageBuilder::escape_html(merged_by));
        }

        if let Some(commit) = &self.merge_commit {
            let short = &commit[..7.min(commit.len())];
            builder = builder.section("🔐 Merge commit", &format!("<code>{}</code>", short));
        }

        builder = builder
            .section("👥 Назначены", &assignees_text)
            .section_bold("⚡️ Инициатор", &safe_source)
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
            let trimmed = url.trim();
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                builder = builder.section(
                    "🔗 Pull Request",
                    &format!(
                        "<a href=\"{}\">Перейти →</a>",
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

impl DomainEvent for WebhookPullRequestEvent {
    const EVENT_NAME: &'static str = "webhook.pull_request";
}

impl MessageBrokerMessage for WebhookPullRequestEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
