use crate::domain::digest::entities::digest_subscription::DigestSubscription;
use crate::domain::digest::repositories::digest_subscription_repository::{
    CreateDigestSubscriptionError, DeleteDigestSubscriptionError, DigestSubscriptionRepository,
    FindDigestSubscriptionError, UpdateDigestSubscriptionError,
};
use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;
use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::digest_subscriptions;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct MySQLDigestSubscriptionRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLDigestSubscriptionRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: digest_subscriptions::Model) -> Option<DigestSubscription> {
        let digest_type = DigestType::from_str(&model.digest_type)?;

        Some(DigestSubscription {
            id: DigestSubscriptionId(model.id),
            user_id: UserId(model.user_id),
            repository_id: model.repository_id.map(RepositoryId),
            digest_type,
            send_hour: model.send_hour,
            send_minute: model.send_minute,
            day_of_week: model.day_of_week,
            is_active: model.is_active != 0,
            last_sent_at: model.last_sent_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        })
    }
}

#[async_trait]
impl DigestSubscriptionRepository for MySQLDigestSubscriptionRepository {
    async fn create(
        &self,
        subscription: &DigestSubscription,
    ) -> Result<DigestSubscription, CreateDigestSubscriptionError> {
        let model = digest_subscriptions::ActiveModel {
            user_id: Set(subscription.user_id.0),
            repository_id: Set(subscription.repository_id.map(|r| r.0)),
            digest_type: Set(subscription.digest_type.as_str().to_string()),
            send_hour: Set(subscription.send_hour),
            send_minute: Set(subscription.send_minute),
            day_of_week: Set(subscription.day_of_week),
            is_active: Set(subscription.is_active as i8),
            last_sent_at: Set(subscription.last_sent_at),
            ..Default::default()
        };

        let result = model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| CreateDigestSubscriptionError::DbError(e.to_string()))?;

        Self::from_mysql(result).ok_or_else(|| {
            CreateDigestSubscriptionError::DbError("Invalid digest_type".to_string())
        })
    }

    async fn find_by_id(
        &self,
        id: DigestSubscriptionId,
    ) -> Result<DigestSubscription, FindDigestSubscriptionError> {
        let model = digest_subscriptions::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindDigestSubscriptionError::DbError(e.to_string()))?
            .ok_or(FindDigestSubscriptionError::NotFound)?;

        Self::from_mysql(model)
            .ok_or_else(|| FindDigestSubscriptionError::DbError("Invalid digest_type".to_string()))
    }

    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<DigestSubscription>, FindDigestSubscriptionError> {
        let models = digest_subscriptions::Entity::find()
            .filter(digest_subscriptions::Column::UserId.eq(user_id.0))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindDigestSubscriptionError::DbError(e.to_string()))?;

        let subscriptions = models.into_iter().filter_map(Self::from_mysql).collect();

        Ok(subscriptions)
    }

    async fn find_due(
        &self,
        hour: i8,
        minute: i8,
    ) -> Result<Vec<DigestSubscription>, FindDigestSubscriptionError> {
        let models = digest_subscriptions::Entity::find()
            .filter(digest_subscriptions::Column::IsActive.eq(1_i8))
            .filter(digest_subscriptions::Column::SendHour.eq(hour))
            .filter(digest_subscriptions::Column::SendMinute.eq(minute))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindDigestSubscriptionError::DbError(e.to_string()))?;

        let subscriptions = models.into_iter().filter_map(Self::from_mysql).collect();

        Ok(subscriptions)
    }

    async fn update(
        &self,
        subscription: &DigestSubscription,
    ) -> Result<DigestSubscription, UpdateDigestSubscriptionError> {
        let model = digest_subscriptions::Entity::find_by_id(subscription.id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpdateDigestSubscriptionError::DbError(e.to_string()))?
            .ok_or(UpdateDigestSubscriptionError::NotFound)?;

        let mut active_model: digest_subscriptions::ActiveModel = model.into();

        active_model.repository_id = Set(subscription.repository_id.map(|r| r.0));
        active_model.digest_type = Set(subscription.digest_type.as_str().to_string());
        active_model.send_hour = Set(subscription.send_hour);
        active_model.send_minute = Set(subscription.send_minute);
        active_model.day_of_week = Set(subscription.day_of_week);
        active_model.is_active = Set(subscription.is_active as i8);
        active_model.last_sent_at = Set(subscription.last_sent_at);

        let result = active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| UpdateDigestSubscriptionError::DbError(e.to_string()))?;

        Self::from_mysql(result).ok_or_else(|| {
            UpdateDigestSubscriptionError::DbError("Invalid digest_type".to_string())
        })
    }

    async fn delete(&self, id: DigestSubscriptionId) -> Result<(), DeleteDigestSubscriptionError> {
        let model = digest_subscriptions::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| DeleteDigestSubscriptionError::DbError(e.to_string()))?
            .ok_or(DeleteDigestSubscriptionError::NotFound)?;

        let active_model: digest_subscriptions::ActiveModel = model.into();

        active_model
            .delete(self.db.as_ref())
            .await
            .map_err(|e| DeleteDigestSubscriptionError::DbError(e.to_string()))?;

        Ok(())
    }
}
