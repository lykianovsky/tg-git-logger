use crate::domain::repository::value_objects::repository_id::RepositoryId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryTaskTracker {
    pub id: i32,
    pub repository_id: RepositoryId,
    pub space_id: i32,
    pub qa_column_id: i32,
    pub extract_pattern_regexp: String,
    pub path_to_card: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
