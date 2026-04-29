use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: RepositoryId,
    pub name: String,
    pub owner: String,
    pub url: String,
    /// Chat ID where raw GitHub webhook notifications go (push / release / workflow / большая PR-карточка).
    /// `None` means fallback to the global `TELEGRAM_CHAT_ID`.
    pub social_chat_id: Option<SocialChatId>,
    /// Chat ID where curated team-relevant notifications go (теги ревьюеров, cc-mentions, approve, stale digest, релизы).
    /// `None` means fallback to `social_chat_id`.
    pub notifications_chat_id: Option<SocialChatId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
