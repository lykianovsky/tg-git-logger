use crate::domain::shared::events::event::DomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

/// A comment on a pull request (includes both review comments and issue comments on PRs).
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPrCommentEvent {
    /// GitHub login of the person who left the comment.
    pub commenter: String,
    /// GitHub login of the PR author (notification recipient).
    pub pr_author: String,
    /// Repository full name, e.g. "owner/repo".
    pub repo: String,
    /// PR number.
    pub pr_number: u64,
    /// PR title.
    pub pr_title: String,
    /// The comment text body.
    pub comment_body: String,
    /// Direct link to the comment on GitHub.
    pub comment_url: String,
}

impl WebhookEvent for WebhookPrCommentEvent {
    fn build_text(&self) -> String {
        MessageBuilder::new()
            .bold("💬 Новый комментарий к вашему PR")
            .empty_line()
            .section_bold("👤 От", &self.commenter)
            .section(
                "📝 PR",
                &format!(
                    "#{} — {}",
                    self.pr_number,
                    MessageBuilder::escape_html(&self.pr_title),
                ),
            )
            .section("📦 Репозиторий", &self.repo)
            .empty_line()
            .bold("Комментарий:")
            .line(&MessageBuilder::escape_html(&self.comment_body))
            .empty_line()
            .section(
                "🔗 Перейти",
                &format!("<a href=\"{}\">Открыть комментарий →</a>", self.comment_url),
            )
            .build()
    }
}

impl DomainEvent for WebhookPrCommentEvent {
    const EVENT_NAME: &'static str = "webhook.pr_comment";
}

impl MessageBrokerMessage for WebhookPrCommentEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
