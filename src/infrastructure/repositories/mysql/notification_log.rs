use crate::domain::notification_log::repositories::notification_log_repository::{
    NotificationLogError, NotificationLogRepository,
};
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::notification_log;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use std::sync::Arc;

pub struct MySQLNotificationLogRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLNotificationLogRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl NotificationLogRepository for MySQLNotificationLogRepository {
    async fn was_sent_within(
        &self,
        user_id: UserId,
        kind: &str,
        key: &str,
        since: DateTime<Utc>,
    ) -> Result<bool, NotificationLogError> {
        let count = notification_log::Entity::find()
            .filter(notification_log::Column::UserId.eq(user_id.0))
            .filter(notification_log::Column::Kind.eq(kind))
            .filter(notification_log::Column::DedupKey.eq(key))
            .filter(notification_log::Column::SentAt.gte(since))
            .count(self.db.as_ref())
            .await
            .map_err(|e| NotificationLogError::DbError(e.to_string()))?;

        Ok(count > 0)
    }

    async fn record_sent(
        &self,
        user_id: UserId,
        kind: &str,
        key: &str,
    ) -> Result<(), NotificationLogError> {
        let active = notification_log::ActiveModel {
            user_id: Set(user_id.0),
            kind: Set(kind.to_string()),
            dedup_key: Set(key.to_string()),
            sent_at: Set(Utc::now()),
            ..Default::default()
        };
        active
            .insert(self.db.as_ref())
            .await
            .map_err(|e| NotificationLogError::DbError(e.to_string()))?;

        Ok(())
    }
}
