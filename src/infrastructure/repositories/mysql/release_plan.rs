use crate::domain::release_plan::entities::release_plan::{
    NewReleasePlan, ReleasePlan, ReleasePlanNotificationKind,
};
use crate::domain::release_plan::repositories::release_plan_repository::{
    CreateReleasePlanError, FindReleasePlanError, ReleasePlanRepository, UpdateReleasePlanError,
};
use crate::domain::release_plan::value_objects::release_plan_id::ReleasePlanId;
use crate::domain::release_plan::value_objects::release_plan_status::ReleasePlanStatus;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::{release_plan_repositories, release_plans};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use std::sync::Arc;

pub struct MySQLReleasePlanRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLReleasePlanRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    async fn load_repository_ids(&self, plan_id: i32) -> Result<Vec<RepositoryId>, sea_orm::DbErr> {
        let rows = release_plan_repositories::Entity::find()
            .filter(release_plan_repositories::Column::ReleasePlanId.eq(plan_id))
            .all(self.db.as_ref())
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| RepositoryId(r.repository_id))
            .collect())
    }

    fn from_mysql(model: release_plans::Model, repo_ids: Vec<RepositoryId>) -> ReleasePlan {
        ReleasePlan {
            id: ReleasePlanId(model.id),
            planned_date: model.planned_date,
            call_datetime: model.call_datetime,
            meeting_url: model.meeting_url,
            note: model.note,
            status: ReleasePlanStatus::from_str(&model.status)
                .unwrap_or(ReleasePlanStatus::Planned),
            announce_chat_id: model.announce_chat_id.map(SocialChatId),
            repository_ids: repo_ids,
            created_by_user_id: UserId(model.created_by_user_id),
            notified_24h_at: model.notified24h_at,
            notified_call_at: model.notified_call_at,
            notified_release_day_at: model.notified_release_day_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl ReleasePlanRepository for MySQLReleasePlanRepository {
    async fn create(&self, plan: &NewReleasePlan) -> Result<ReleasePlan, CreateReleasePlanError> {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| CreateReleasePlanError::DbError(e.to_string()))?;

        let active = release_plans::ActiveModel {
            planned_date: Set(plan.planned_date),
            call_datetime: Set(plan.call_datetime),
            meeting_url: Set(plan.meeting_url.clone()),
            note: Set(plan.note.clone()),
            status: Set(ReleasePlanStatus::Planned.as_str().to_string()),
            announce_chat_id: Set(plan.announce_chat_id.map(|c| c.0)),
            created_by_user_id: Set(plan.created_by_user_id.0),
            ..Default::default()
        };

        let inserted = active
            .insert(&txn)
            .await
            .map_err(|e| CreateReleasePlanError::DbError(e.to_string()))?;

        for repo_id in &plan.repository_ids {
            let link = release_plan_repositories::ActiveModel {
                release_plan_id: Set(inserted.id),
                repository_id: Set(repo_id.0),
            };
            link.insert(&txn)
                .await
                .map_err(|e| CreateReleasePlanError::DbError(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| CreateReleasePlanError::DbError(e.to_string()))?;

        Ok(Self::from_mysql(inserted, plan.repository_ids.clone()))
    }

    async fn find_by_id(&self, id: ReleasePlanId) -> Result<ReleasePlan, FindReleasePlanError> {
        let model = release_plans::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindReleasePlanError::DbError(e.to_string()))?
            .ok_or(FindReleasePlanError::NotFound)?;
        let repo_ids = self
            .load_repository_ids(model.id)
            .await
            .map_err(|e| FindReleasePlanError::DbError(e.to_string()))?;
        Ok(Self::from_mysql(model, repo_ids))
    }

    async fn find_active(&self) -> Result<Vec<ReleasePlan>, FindReleasePlanError> {
        let models = release_plans::Entity::find()
            .filter(release_plans::Column::Status.eq("planned"))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindReleasePlanError::DbError(e.to_string()))?;

        let mut result = Vec::with_capacity(models.len());
        for model in models {
            let repo_ids = self
                .load_repository_ids(model.id)
                .await
                .map_err(|e| FindReleasePlanError::DbError(e.to_string()))?;
            result.push(Self::from_mysql(model, repo_ids));
        }
        Ok(result)
    }

    async fn set_status(
        &self,
        id: ReleasePlanId,
        status: ReleasePlanStatus,
    ) -> Result<(), UpdateReleasePlanError> {
        let model = release_plans::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpdateReleasePlanError::DbError(e.to_string()))?
            .ok_or_else(|| UpdateReleasePlanError::DbError("Not found".to_string()))?;
        let mut active: release_plans::ActiveModel = model.into();
        active.status = Set(status.as_str().to_string());
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| UpdateReleasePlanError::DbError(e.to_string()))?;
        Ok(())
    }

    async fn mark_notified(
        &self,
        id: ReleasePlanId,
        kind: ReleasePlanNotificationKind,
        at: DateTime<Utc>,
    ) -> Result<(), UpdateReleasePlanError> {
        let model = release_plans::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpdateReleasePlanError::DbError(e.to_string()))?
            .ok_or_else(|| UpdateReleasePlanError::DbError("Not found".to_string()))?;
        let mut active: release_plans::ActiveModel = model.into();
        match kind {
            ReleasePlanNotificationKind::Day24h => active.notified24h_at = Set(Some(at)),
            ReleasePlanNotificationKind::Call => active.notified_call_at = Set(Some(at)),
            ReleasePlanNotificationKind::ReleaseDay => {
                active.notified_release_day_at = Set(Some(at))
            }
        }
        active
            .update(self.db.as_ref())
            .await
            .map_err(|e| UpdateReleasePlanError::DbError(e.to_string()))?;
        Ok(())
    }
}
