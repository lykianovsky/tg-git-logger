use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use chrono::{DateTime, NaiveDate, Utc};

pub struct CreateReleasePlanExecutorCommand {
    pub created_by_social_user_id: SocialUserId,
    pub planned_date: NaiveDate,
    pub call_datetime: Option<DateTime<Utc>>,
    pub meeting_url: Option<String>,
    pub note: Option<String>,
    pub announce_chat_id: Option<SocialChatId>,
    pub repository_ids: Vec<RepositoryId>,
}
