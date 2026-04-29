use crate::domain::user::value_objects::user_id::UserId;
use crate::domain::user_preferences::entities::user_preferences::UserPreferences;
use crate::domain::user_preferences::repositories::user_preferences_repository::{
    FindUserPreferencesError, UpsertUserPreferencesError, UserPreferencesRepository,
};
use crate::domain::user_preferences::value_objects::notification_event_kind::NotificationEventKind;
use crate::domain::user_preferences::value_objects::quiet_hours_window::QuietHoursWindow;
use crate::domain::user_preferences::value_objects::user_preferences_id::UserPreferencesId;
use crate::infrastructure::database::mysql::entities::user_preferences;
use async_trait::async_trait;
use chrono_tz::Tz;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct MySQLUserPreferencesRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserPreferencesRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: user_preferences::Model) -> UserPreferences {
        let timezone: Option<Tz> = model.timezone.as_deref().and_then(|s| s.parse::<Tz>().ok());

        let dnd_window = match (model.dnd_start, model.dnd_end) {
            (Some(start), Some(end)) => Some(QuietHoursWindow::new(start, end)),
            _ => None,
        };

        let enabled_events = parse_enabled_events(&model.enabled_events);

        UserPreferences {
            id: UserPreferencesId(model.id),
            user_id: UserId(model.user_id),
            timezone,
            dnd_window,
            vacation_until: model.vacation_until,
            snooze_until: model.snooze_until,
            enabled_events,
            priority_only: model.priority_only != 0,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

fn parse_enabled_events(json: &serde_json::Value) -> Vec<NotificationEventKind> {
    json.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(NotificationEventKind::from_str)
                .collect()
        })
        .unwrap_or_else(NotificationEventKind::all_default_enabled)
}

fn serialize_enabled_events(events: &[NotificationEventKind]) -> serde_json::Value {
    serde_json::Value::Array(
        events
            .iter()
            .map(|e| serde_json::Value::String(e.as_str().to_string()))
            .collect(),
    )
}

#[async_trait]
impl UserPreferencesRepository for MySQLUserPreferencesRepository {
    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Option<UserPreferences>, FindUserPreferencesError> {
        let model = user_preferences::Entity::find()
            .filter(user_preferences::Column::UserId.eq(user_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserPreferencesError::DbError(e.to_string()))?;

        Ok(model.map(Self::from_mysql))
    }

    async fn upsert(
        &self,
        prefs: &UserPreferences,
    ) -> Result<UserPreferences, UpsertUserPreferencesError> {
        let timezone_str = prefs.timezone.map(|tz| tz.name().to_string());
        let dnd_start = prefs.dnd_window.map(|w| w.start);
        let dnd_end = prefs.dnd_window.map(|w| w.end);
        let enabled_events_json = serialize_enabled_events(&prefs.enabled_events);

        let existing = user_preferences::Entity::find()
            .filter(user_preferences::Column::UserId.eq(prefs.user_id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpsertUserPreferencesError::DbError(e.to_string()))?;

        let result = match existing {
            Some(model) => {
                let mut active: user_preferences::ActiveModel = model.into();
                active.timezone = Set(timezone_str);
                active.dnd_start = Set(dnd_start);
                active.dnd_end = Set(dnd_end);
                active.vacation_until = Set(prefs.vacation_until);
                active.snooze_until = Set(prefs.snooze_until);
                active.enabled_events = Set(enabled_events_json);
                active.priority_only = Set(prefs.priority_only as i8);
                active
                    .update(self.db.as_ref())
                    .await
                    .map_err(|e| UpsertUserPreferencesError::DbError(e.to_string()))?
            }
            None => {
                let active = user_preferences::ActiveModel {
                    user_id: Set(prefs.user_id.0),
                    timezone: Set(timezone_str),
                    dnd_start: Set(dnd_start),
                    dnd_end: Set(dnd_end),
                    vacation_until: Set(prefs.vacation_until),
                    snooze_until: Set(prefs.snooze_until),
                    enabled_events: Set(enabled_events_json),
                    priority_only: Set(prefs.priority_only as i8),
                    ..Default::default()
                };
                active
                    .insert(self.db.as_ref())
                    .await
                    .map_err(|e| UpsertUserPreferencesError::DbError(e.to_string()))?
            }
        };

        Ok(Self::from_mysql(result))
    }
}
