use crate::domain::pending_notification::entities::pending_notification::PendingNotification;
use crate::domain::pending_notification::repositories::pending_notification_repository::{
    CreatePendingNotificationError, DeletePendingNotificationError, FindPendingNotificationError,
    PendingNotificationsRepository,
};
use crate::domain::pending_notification::value_objects::pending_notification_id::PendingNotificationId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::pending_notifications;
use crate::utils::builder::message::MessageBuilder;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLPendingNotificationsRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLPendingNotificationsRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: pending_notifications::Model) -> Result<PendingNotification, String> {
        let social_type = SocialType::from_str(&model.social_type).map_err(|e| {
            format!(
                "Invalid social_type in DB: {}, error: {:?}",
                model.social_type, e
            )
        })?;

        let message: MessageBuilder = serde_json::from_value(model.payload.clone())
            .map_err(|e| format!("Invalid payload in pending_notifications: {e}"))?;

        Ok(PendingNotification {
            id: PendingNotificationId(model.id),
            user_id: model.user_id.map(UserId),
            social_type,
            social_chat_id: SocialChatId(model.social_chat_id),
            message,
            event_type: model.event_type,
            deliver_after: model.deliver_after,
            created_at: model.created_at,
        })
    }
}

#[async_trait]
impl PendingNotificationsRepository for MySQLPendingNotificationsRepository {
    async fn create(
        &self,
        notification: &PendingNotification,
    ) -> Result<PendingNotification, CreatePendingNotificationError> {
        let payload_json = serde_json::to_value(&notification.message)
            .map_err(|e| CreatePendingNotificationError::DbError(e.to_string()))?;

        let active = pending_notifications::ActiveModel {
            user_id: Set(notification.user_id.map(|u| u.0)),
            social_type: Set(notification.social_type.to_string()),
            social_chat_id: Set(notification.social_chat_id.0),
            payload: Set(payload_json),
            event_type: Set(notification.event_type.clone()),
            deliver_after: Set(notification.deliver_after),
            ..Default::default()
        };

        let result = active
            .insert(self.db.as_ref())
            .await
            .map_err(|e| CreatePendingNotificationError::DbError(e.to_string()))?;

        Self::from_mysql(result).map_err(CreatePendingNotificationError::DbError)
    }

    async fn find_due(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<PendingNotification>, FindPendingNotificationError> {
        let models = pending_notifications::Entity::find()
            .filter(pending_notifications::Column::DeliverAfter.lte(now))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindPendingNotificationError::DbError(e.to_string()))?;

        let notifications = models
            .into_iter()
            .filter_map(|m| match Self::from_mysql(m) {
                Ok(n) => Some(n),
                Err(e) => {
                    tracing::warn!(error = %e, "Skipping malformed pending_notification row");
                    None
                }
            })
            .collect();

        Ok(notifications)
    }

    async fn delete_many(
        &self,
        ids: &[PendingNotificationId],
    ) -> Result<(), DeletePendingNotificationError> {
        if ids.is_empty() {
            return Ok(());
        }
        let id_values: Vec<i32> = ids.iter().map(|i| i.0).collect();

        pending_notifications::Entity::delete_many()
            .filter(pending_notifications::Column::Id.is_in(id_values))
            .exec(self.db.as_ref())
            .await
            .map_err(|e| DeletePendingNotificationError::DbError(e.to_string()))?;

        Ok(())
    }
}
