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
    #[serde(default)]
    pub mergeable_state: Option<String>, // GitHub: clean / dirty / unknown / blocked / behind / draft
}

impl WebhookEvent for WebhookPullRequestReviewEvent {
    fn build_text(&self) -> String {
        let (icon, heading) = match self.state {
            WebhookPullRequestReviewState::Approved => ("✅", "PR одобрен"),
            WebhookPullRequestReviewState::ChangesRequested => ("🔄", "Запрошены изменения"),
            WebhookPullRequestReviewState::Commented => ("💬", "Оставлен комментарий к PR"),
            WebhookPullRequestReviewState::Unknown => ("ℹ️", "Ревью PR"),
        };

        let pr_url_trimmed = self.pr_url.trim();
        let pr_link = if pr_url_trimmed.starts_with("http://")
            || pr_url_trimmed.starts_with("https://")
        {
            format!(
                "<a href=\"{}\">{}</a>",
                MessageBuilder::escape_html(pr_url_trimmed),
                MessageBuilder::escape_html(&self.pr_title)
            )
        } else {
            MessageBuilder::escape_html(&self.pr_title)
        };

        let mut builder = MessageBuilder::new()
            .bold(&format!("{} {} — PR #{}", icon, heading, self.pr_number))
            .empty_line()
            .section("📝 PR", &pr_link)
            .section_bold("👤 Ревьюер", &MessageBuilder::escape_html(&self.reviewer))
            .section("📦 Репозиторий", &MessageBuilder::escape_html(&self.repo));

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

        let review_url_trimmed = self.review_url.trim();
        let review_link = if review_url_trimmed.starts_with("http://")
            || review_url_trimmed.starts_with("https://")
        {
            format!(
                "<a href=\"{}\">Открыть →</a>",
                MessageBuilder::escape_html(review_url_trimmed)
            )
        } else {
            "—".to_string()
        };

        builder
            .empty_line()
            .section("🔗 Ревью", &review_link)
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
