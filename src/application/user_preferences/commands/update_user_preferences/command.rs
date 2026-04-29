use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use chrono::{DateTime, NaiveTime, Utc};
use chrono_tz::Tz;

pub enum UserPreferencesPatch {
    SetDndWindow {
        start: NaiveTime,
        end: NaiveTime,
    },
    ClearDndWindow,
    SetTimezone {
        timezone: Tz,
    },
    ClearTimezone,
    SetVacation {
        until: DateTime<Utc>,
    },
    ClearVacation,
    SetSnooze {
        until: DateTime<Utc>,
    },
    ClearSnooze,
    ToggleEnabledEvent {
        event: NotificationEventKind,
        enabled: bool,
    },
    SetPriorityOnly {
        enabled: bool,
    },
    Reset,
}

pub struct UpdateUserPreferencesExecutorCommand {
    pub social_user_id: SocialUserId,
    pub patch: UserPreferencesPatch,
}
