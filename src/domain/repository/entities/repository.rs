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
    /// Chat ID where webhook notifications for this repository are sent.
    /// `None` means notifications fall back to the global `TELEGRAM_CHAT_ID`.
    pub social_chat_id: Option<SocialChatId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
