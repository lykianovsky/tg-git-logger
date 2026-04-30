use crate::domain::repository::entities::repository::Repository;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

pub struct GetUserOverviewResponse {
    pub github_login: Option<String>,
    pub roles: Vec<RoleName>,
    pub dnd_window: Option<QuietHoursWindow>,
    pub timezone: Option<Tz>,
    pub vacation_until: Option<DateTime<Utc>>,
    pub snooze_until: Option<DateTime<Utc>>,
    pub repositories: Vec<Repository>,
}
