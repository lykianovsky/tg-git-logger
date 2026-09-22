use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use crate::domain::user::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};

/// Карточка в трекере, заведённая по упавшему тесту. Одна активная на тест: повторное
/// нажатие показывает существующую, а новая создаётся только если прошлая закрыта.
#[derive(Debug, Clone)]
pub struct TestFailureCard {
    pub id: i32,
    pub repository_id: RepositoryId,
    pub fingerprint: TestFingerprint,
    pub project: String,
    pub file: String,
    pub title: String,
    pub card_id: u64,
    pub card_url: String,
    pub previous_card_id: Option<u64>,
    pub created_by_user_id: Option<UserId>,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct NewTestFailureCard {
    pub repository_id: RepositoryId,
    pub fingerprint: TestFingerprint,
    pub project: String,
    pub file: String,
    pub title: String,
    pub card_id: u64,
    pub card_url: String,
    pub previous_card_id: Option<u64>,
    pub created_by_user_id: Option<UserId>,
}
