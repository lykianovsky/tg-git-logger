use crate::domain::health_ping::entities::health_ping::HealthPing;
use crate::domain::health_ping::repositories::health_ping_repository::{
    CreateHealthPingError, DeleteHealthPingError, FindHealthPingError, HealthPingRepository,
    UpdateHealthPingError,
};
use crate::domain::health_ping::value_objects::health_ping_id::HealthPingId;
use crate::infrastructure::database::mysql::entities::health_pings;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct MySQLHealthPingRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLHealthPingRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: health_pings::Model) -> HealthPing {
        HealthPing {
            id: HealthPingId(model.id),
            name: model.name,
            url: model.url,
            interval_minutes: model.interval_minutes,
            is_active: model.is_active != 0,
            last_checked_at: model.last_checked_at,
            last_status: model.last_status,
            last_response_ms: model.last_response_ms,
            last_error_message: model.last_error_message,
            failed_since: model.failed_since,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl HealthPingRepository for MySQLHealthPingRepository {
    async fn create(&self, ping: &HealthPing) -> Result<HealthPing, CreateHealthPingError> {
        let model = health_pings::ActiveModel {
            name: Set(ping.name.clone()),
            url: Set(ping.url.clone()),
            interval_minutes: Set(ping.interval_minutes),
            is_active: Set(ping.is_active as i8),
            ..Default::default()
        };

        let result = model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| CreateHealthPingError::DbError(e.to_string()))?;

        Ok(Self::from_mysql(result))
    }

    async fn find_by_id(&self, id: HealthPingId) -> Result<HealthPing, FindHealthPingError> {
        let model = health_pings::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindHealthPingError::DbError(e.to_string()))?
            .ok_or(FindHealthPingError::NotFound)?;

        Ok(Self::from_mysql(model))
    }

    async fn find_all(&self) -> Result<Vec<HealthPing>, FindHealthPingError> {
        let models = health_pings::Entity::find()
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindHealthPingError::DbError(e.to_string()))?;

        Ok(models.into_iter().map(Self::from_mysql).collect())
    }

    async fn find_active_due(&self) -> Result<Vec<HealthPing>, FindHealthPingError> {
        let models = health_pings::Entity::find()
            .filter(health_pings::Column::IsActive.eq(1_i8))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindHealthPingError::DbError(e.to_string()))?;

        let now = Utc::now();

        let due: Vec<HealthPing> = models
            .into_iter()
            .map(Self::from_mysql)
            .filter(|p| match p.last_checked_at {
                None => true,
                Some(last) => {
                    let elapsed = now.signed_duration_since(last);
                    elapsed.num_minutes() >= p.interval_minutes as i64
                }
            })
            .collect();

        Ok(due)
    }

    async fn update(&self, ping: &HealthPing) -> Result<HealthPing, UpdateHealthPingError> {
        let model = health_pings::Entity::find_by_id(ping.id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpdateHealthPingError::DbError(e.to_string()))?
            .ok_or(UpdateHealthPingError::NotFound)?;

        let mut active_model: health_pings::ActiveModel = model.into();

        active_model.name = Set(ping.name.clone());
        active_model.url = Set(ping.url.clone());
        active_model.interval_minutes = Set(ping.interval_minutes);
        active_model.is_active = Set(ping.is_active as i8);
        active_model.last_checked_at = Set(ping.last_checked_at);
        active_model.last_status = Set(ping.last_status.clone());
        active_model.last_response_ms = Set(ping.last_response_ms);
        active_model.last_error_message = Set(ping.last_error_message.clone());
        active_model.failed_since = Set(ping.failed_since);

        let result = active_model
            .update(self.db.as_ref())
            .await
            .map_err(|e| UpdateHealthPingError::DbError(e.to_string()))?;

        Ok(Self::from_mysql(result))
    }

    async fn delete(&self, id: HealthPingId) -> Result<(), DeleteHealthPingError> {
        let model = health_pings::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| DeleteHealthPingError::DbError(e.to_string()))?
            .ok_or(DeleteHealthPingError::NotFound)?;

        let active_model: health_pings::ActiveModel = model.into();

        active_model
            .delete(self.db.as_ref())
            .await
            .map_err(|e| DeleteHealthPingError::DbError(e.to_string()))?;

        Ok(())
    }
}
