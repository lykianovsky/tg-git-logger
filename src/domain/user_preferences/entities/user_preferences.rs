use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use crate::domain::user_preferences::value_objects::user_preferences_id::UserPreferencesId;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub id: UserPreferencesId,
    pub user_id: UserId,
    pub timezone: Option<Tz>,
    pub dnd_window: Option<QuietHoursWindow>,
    pub vacation_until: Option<DateTime<Utc>>,
    pub snooze_until: Option<DateTime<Utc>>,
    pub enabled_events: Vec<NotificationEventKind>,
    pub priority_only: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
