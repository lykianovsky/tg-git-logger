use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Отключение тестов у репозитория: настройки удаляются, история прогонов остаётся
pub struct DisconnectTestSuiteCommand {
    pub repository_id: RepositoryId,
}
