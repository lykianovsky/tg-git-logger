use crate::domain::pr_review::entities::pr_review::PrReview;
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpsertPrReviewError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[derive(Debug, Error)]
pub enum FindPrReviewError {
    #[error("Database error: {0}")]
    DbError(String),
}

#[async_trait::async_trait]
pub trait PrReviewRepository: Send + Sync {
    async fn upsert(
        &self,
        repo: &str,
        pr_number: u64,
        reviewer_login: &str,
        state: &str,
        reviewed_at: DateTime<Utc>,
    ) -> Result<(), UpsertPrReviewError>;

    async fn find_by_pr(
        &self,
        repo: &str,
        pr_number: u64,
    ) -> Result<Vec<PrReview>, FindPrReviewError>;
}
