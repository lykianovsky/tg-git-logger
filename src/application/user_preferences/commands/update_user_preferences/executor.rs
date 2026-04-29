use crate::application::user_preferences::commands::update_user_preferences::command::{
    UpdateUserPreferencesExecutorCommand, UserPreferencesPatch,
};
use crate::application::user_preferences::commands::update_user_preferences::error::UpdateUserPreferencesExecutorError;
use crate::application::user_preferences::commands::update_user_preferences::response::UpdateUserPreferencesExecutorResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user_preferences::entities::user_preferences::UserPreferences;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use crate::domain::user_preferences::value_objects::user_preferences_id::UserPreferencesId;
use chrono::Utc;
use std::sync::Arc;

pub struct UpdateUserPreferencesExecutor {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
}

impl UpdateUserPreferencesExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_preferences_repo,
        }
    }
}

impl CommandExecutor for UpdateUserPreferencesExecutor {
    type Command = UpdateUserPreferencesExecutorCommand;
    type Response = UpdateUserPreferencesExecutorResponse;
    type Error = UpdateUserPreferencesExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social_account = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await?;
        let user_id = social_account.user_id;

        let existing = self.user_preferences_repo.find_by_user_id(user_id).await?;

        let mut prefs = existing.unwrap_or_else(|| default_preferences(user_id));

        apply_patch(&mut prefs, &cmd.patch);
        prefs.updated_at = Utc::now();

        let saved = self.user_preferences_repo.upsert(&prefs).await?;

        Ok(UpdateUserPreferencesExecutorResponse { preferences: saved })
    }
}

fn default_preferences(user_id: UserId) -> UserPreferences {
    let now = Utc::now();
    UserPreferences {
        id: UserPreferencesId(0),
        user_id,
        timezone: None,
        dnd_window: None,
        vacation_until: None,
        snooze_until: None,
        enabled_events: NotificationEventKind::all_default_enabled(),
        priority_only: false,
        created_at: now,
        updated_at: now,
    }
}

fn apply_patch(prefs: &mut UserPreferences, patch: &UserPreferencesPatch) {
    match patch {
        UserPreferencesPatch::SetDndWindow { start, end } => {
            prefs.dnd_window = Some(QuietHoursWindow::new(*start, *end));
        }
        UserPreferencesPatch::ClearDndWindow => {
            prefs.dnd_window = None;
        }
        UserPreferencesPatch::SetTimezone { timezone } => {
            prefs.timezone = Some(*timezone);
        }
        UserPreferencesPatch::ClearTimezone => {
            prefs.timezone = None;
        }
        UserPreferencesPatch::SetVacation { until } => {
            prefs.vacation_until = Some(*until);
        }
        UserPreferencesPatch::ClearVacation => {
            prefs.vacation_until = None;
        }
        UserPreferencesPatch::SetSnooze { until } => {
            prefs.snooze_until = Some(*until);
        }
        UserPreferencesPatch::ClearSnooze => {
            prefs.snooze_until = None;
        }
        UserPreferencesPatch::ToggleEnabledEvent { event, enabled } => {
            prefs.enabled_events.retain(|e| e != event);
            if *enabled {
                prefs.enabled_events.push(*event);
            }
        }
        UserPreferencesPatch::SetPriorityOnly { enabled } => {
            prefs.priority_only = *enabled;
        }
        UserPreferencesPatch::Reset => {
            prefs.timezone = None;
            prefs.dnd_window = None;
            prefs.vacation_until = None;
            prefs.snooze_until = None;
            prefs.enabled_events = NotificationEventKind::all_default_enabled();
            prefs.priority_only = false;
        }
    }
}
