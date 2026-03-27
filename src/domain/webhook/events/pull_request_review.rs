use crate::domain::shared::events::event::DomainEvent;
use crate::domain::webhook::events::WebhookEvent;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookPullRequestReviewState {
    Approved,
    ChangesRequested,
    Commented,
    Unknown,
}

/// Fired when a reviewer submits a pull-request review (approved / changes_requested / commented).
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPullRequestReviewEvent {
    pub reviewer: String,  // GitHub login of the reviewer
    pub pr_author: String, // GitHub login of the PR author (notification recipient)
    pub repo: String,      // full repository name, e.g. "owner/repo"
    pub pr_number: u64,
    pub pr_title: String,
    pub pr_url: String,              // direct link to the PR
    pub review_url: String,          // direct link to the review
    pub review_body: Option<String>, // top-level review comment body (None if empty)
    pub state: WebhookPullRequestReviewState,
    pub review_comments: u64, // total review-comment count on the PR at submission time
}

impl WebhookEvent for WebhookPullRequestReviewEvent {
    fn build_text(&self) -> String {
        let (icon, heading) = match self.state {
            WebhookPullRequestReviewState::Approved => ("✅", "PR одобрен"),
            WebhookPullRequestReviewState::ChangesRequested => ("🔄", "Запрошены изменения"),
            WebhookPullRequestReviewState::Commented => ("💬", "Оставлен комментарий к PR"),
            WebhookPullRequestReviewState::Unknown => ("ℹ️", "Ревью PR"),
        };

        let mut builder = MessageBuilder::new()
            .bold(&format!("{} {} — PR #{}", icon, heading, self.pr_number))
            .empty_line()
            .section(
                "📝 PR",
                &format!(
                    "<a href=\"{}\">{}</a>",
                    self.pr_url,
                    MessageBuilder::escape_html(&self.pr_title),
                ),
            )
            .section_bold("👤 Ревьюер", &self.reviewer)
            .section("📦 Репозиторий", &self.repo);

        if self.state == WebhookPullRequestReviewState::ChangesRequested {
            builder = builder.section(
                "💬 Комментариев",
                &format!("<b>{}</b>", self.review_comments),
            );
        }

        if let Some(body) = &self.review_body {
            builder = builder
                .empty_line()
                .bold("Комментарий:")
                .line(&MessageBuilder::escape_html(body));
        }

        builder
            .empty_line()
            .section(
                "🔗 Ревью",
                &format!("<a href=\"{}\">Открыть →</a>", self.review_url),
            )
            .build()
    }
}

impl DomainEvent for WebhookPullRequestReviewEvent {
    const EVENT_NAME: &'static str = "webhook.pull_request_review";
}

impl MessageBrokerMessage for WebhookPullRequestReviewEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
