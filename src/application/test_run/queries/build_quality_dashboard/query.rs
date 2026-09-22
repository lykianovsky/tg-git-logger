use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Состояние качества репозитория: тренд прогонов и что падает чаще всего
pub struct BuildQualityDashboardQuery {
    pub repository_id: RepositoryId,
}
