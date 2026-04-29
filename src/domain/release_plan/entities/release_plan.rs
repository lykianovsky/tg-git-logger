use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
use crate::domain::release_plan::value_objects::release_plan_status::ReleasePlanStatus;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone)]
pub struct ReleasePlan {
    pub id: ReleasePlanId,
    pub planned_date: NaiveDate,
    pub call_datetime: Option<DateTime<Utc>>,
    pub meeting_url: Option<String>,
    pub note: Option<String>,
    pub status: ReleasePlanStatus,
    pub announce_chat_id: Option<SocialChatId>,
    pub repository_ids: Vec<RepositoryId>,
    pub created_by_user_id: UserId,
    pub notified_24h_at: Option<DateTime<Utc>>,
    pub notified_call_at: Option<DateTime<Utc>>,
    pub notified_release_day_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewReleasePlan {
    pub planned_date: NaiveDate,
    pub call_datetime: Option<DateTime<Utc>>,
    pub meeting_url: Option<String>,
    pub note: Option<String>,
    pub announce_chat_id: Option<SocialChatId>,
    pub repository_ids: Vec<RepositoryId>,
    pub created_by_user_id: UserId,
}

#[derive(Debug, Clone, Copy)]
pub enum ReleasePlanNotificationKind {
    Day24h,
    Call,
    ReleaseDay,
}
