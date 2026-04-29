use crate::domain::pr_review::entities::pr_review::PrReview;
use crate::domain::pr_review::repositories::pr_review_repository::{
    FindPrReviewError, PrReviewRepository, UpsertPrReviewError,
};
use crate::domain::pr_review::value_objects::pr_review_id::PrReviewId;
use crate::infrastructure::database::mysql::entities::pr_reviews;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct MySQLPrReviewRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLPrReviewRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: pr_reviews::Model) -> PrReview {
        PrReview {
            id: PrReviewId(model.id),
            repo: model.repo,
            pr_number: model.pr_number as u64,
            reviewer_login: model.reviewer_login,
            last_reviewed_at: model.last_reviewed_at,
            last_review_state: model.last_review_state,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[async_trait]
impl PrReviewRepository for MySQLPrReviewRepository {
    async fn upsert(
        &self,
        repo: &str,
        pr_number: u64,
        reviewer_login: &str,
        state: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), UpsertPrReviewError> {
        let existing = pr_reviews::Entity::find()
            .filter(pr_reviews::Column::Repo.eq(repo))
            .filter(pr_reviews::Column::PrNumber.eq(pr_number as i32))
            .filter(pr_reviews::Column::ReviewerLogin.eq(reviewer_login))
            .one(self.db.as_ref())
            .await
            .map_err(|e| UpsertPrReviewError::DbError(e.to_string()))?;

        match existing {
            Some(model) => {
                let mut active: pr_reviews::ActiveModel = model.into();
                active.last_reviewed_at = Set(reviewed_at);
                active.last_review_state = Set(state.to_string());
                active
                    .update(self.db.as_ref())
                    .await
                    .map_err(|e| UpsertPrReviewError::DbError(e.to_string()))?;
            }
            None => {
                let active = pr_reviews::ActiveModel {
                    repo: Set(repo.to_string()),
                    pr_number: Set(pr_number as i32),
                    reviewer_login: Set(reviewer_login.to_string()),
                    last_reviewed_at: Set(reviewed_at),
                    last_review_state: Set(state.to_string()),
                    ..Default::default()
                };
                active
                    .insert(self.db.as_ref())
                    .await
                    .map_err(|e| UpsertPrReviewError::DbError(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn find_by_pr(
        &self,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<PrReview>, FindPrReviewError> {
        let models = pr_reviews::Entity::find()
            .filter(pr_reviews::Column::Repo.eq(repo))
            .filter(pr_reviews::Column::PrNumber.eq(pr_number as i32))
            .all(self.db.as_ref())
            .await
            .map_err(|e| FindPrReviewError::DbError(e.to_string()))?;

        Ok(models.into_iter().map(Self::from_mysql).collect())
    }
}
