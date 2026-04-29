use crate::domain::user_preferences::entities::user_preferences::UserPreferences;

pub struct GetUserPreferencesResponse {
    pub preferences: Option<UserPreferences>,
}
