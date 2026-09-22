use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Готовность к релизу по последнему прогону репозитория
pub struct GetReleaseReadinessQuery {
    pub repository_id: RepositoryId,
}
