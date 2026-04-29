pub mod pr_comment;
pub mod pr_conflict;
pub mod pr_mentions;
pub mod pr_opened_tag_reviewers;
pub mod pull_request;
pub mod pull_request_review;
pub mod push;
pub mod re_review_nudge;
pub mod release;
pub mod review_requested;
pub mod workflow;

use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use std::sync::Arc;

/// Resolve the Telegram chat_id to notify for a given GitHub repository.
///
/// Parses `full_name` as `owner/repo`, looks up the repository record, and
/// returns its `telegram_chat_id` if configured.  Falls back to
/// `default_chat_id` when the repository is not found or has no chat
/// configured.
pub async fn resolve_chat_id(
    repository_repo: &Arc<dyn RepositoryRepository>,
    full_name: &str,
    default_chat_id: SocialChatId,
) -> SocialChatId {
    let mut parts = full_name.splitn(2, '/');
    let (owner, name) = match (parts.next(), parts.next()) {
        (Some(o), Some(n)) => (o, n),
        _ => return default_chat_id,
    };

    match repository_repo.find_by_owner_and_name(owner, name).await {
        Ok(repo) => repo.social_chat_id.unwrap_or(default_chat_id),
        Err(_) => default_chat_id,
    }
}

/// Resolve the chat for curated team notifications (теги ревьюеров, cc, approve, релизы, stale digest).
/// Priority: `notifications_chat_id` → `social_chat_id` → default.
pub async fn resolve_notifications_chat_id(
    repository_repo: &Arc<dyn RepositoryRepository>,
    full_name: &str,
    default_chat_id: SocialChatId,
) -> SocialChatId {
    let mut parts = full_name.splitn(2, '/');
    let (owner, name) = match (parts.next(), parts.next()) {
        (Some(o), Some(n)) => (o, n),
        _ => return default_chat_id,
    };

    match repository_repo.find_by_owner_and_name(owner, name).await {
        Ok(repo) => repo
            .notifications_chat_id
            .or(repo.social_chat_id)
            .unwrap_or(default_chat_id),
        Err(_) => default_chat_id,
    }
}
