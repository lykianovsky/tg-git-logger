use crate::domain::user::entities::user_notification::UserNotification;
use crate::domain::user::repositories::user_notifications_repository::{
    CreateUserNotificationError, FindUserNotificationByIdError, FindUserNotificationsByUserIdError,
    UserNotificationsRepository,
};
use crate::domain::user::value_objects::notification_type::NotificationType;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::user_notifications;
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, Set,
};
use std::str::FromStr;
use std::sync::Arc;

pub struct MySQLUserNotificationsRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserNotificationsRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserNotificationsRepository for MySQLUserNotificationsRepository {
    async fn create(
        &self,
        txn: &DatabaseTransaction,
        notification: &UserNotification,
    ) -> Result<UserNotification, CreateUserNotificationError> {
        let model = user_notifications::ActiveModel {
            user_id: Set(notification.user_id.0),
            notification_type: Set(notification.notification_type.to_string()),
            interval_minutes: Set(notification.interval_minutes),
            is_active: Set(notification.is_active as i8),
            last_notified_at: Set(notification.last_notified_at),
            ..Default::default()
        };

        let result = model
            .insert(txn)
            .await
            .map_err(|e| CreateUserNotificationError::DbError(e.to_string()))?;

        UserNotification::from_mysql(result).map_err(CreateUserNotificationError::DbError)
    }

    async fn find_by_id(&self, id: i32) -> Result<UserNotification, FindUserNotificationByIdError> {
        let result = user_notifications::Entity::find()
            .filter(user_notifications::Column::Id.eq(id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserNotificationByIdError::DbError(e.to_string()))?
            .ok_or(FindUserNotificationByIdError::NotFound)?;

        UserNotification::from_mysql(result).map_err(FindUserNotificationByIdError::DbError)
    }

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<UserNotification>, FindUserNotificationsByUserIdError> {
        let results = user_notifications::Entity::find()
            .filter(user_notifications::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindUserNotificationsByUserIdError::DbError(e.to_string()))?;

        results
            .into_iter()
            .map(|m| {
                UserNotification::from_mysql(m).map_err(FindUserNotificationsByUserIdError::DbError)
            })
            .collect()
    }
}

impl UserNotification {
    pub fn from_mysql(model: user_notifications::Model) -> Result<Self, String> {
        let notification_type = NotificationType::from_str(&model.notification_type)
            .map_err(|e| format!("Invalid notification_type in DB: {}", e))?;

        Ok(Self {
            id: model.id,
            user_id: UserId(model.user_id),
            notification_type,
            interval_minutes: model.interval_minutes,
            is_active: model.is_active != 0,
            last_notified_at: model.last_notified_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}
