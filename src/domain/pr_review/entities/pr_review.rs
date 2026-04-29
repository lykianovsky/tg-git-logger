use crate::domain::pr_review::value_objects::pr_review_id::PrReviewId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct PrReview {
    pub id: PrReviewId,
    pub repo: String,
    pub pr_number: u64,
    pub reviewer_login: String,
    pub last_reviewed_at: DateTime<Utc>,
    pub last_review_state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
