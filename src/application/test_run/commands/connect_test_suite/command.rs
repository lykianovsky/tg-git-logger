use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Подключение репозитория к тестам: спрашиваем только процесс CI и ветку,
/// остальное берём из значений по умолчанию
pub struct ConnectTestSuiteCommand {
    pub repository_id: RepositoryId,
    pub workflow_file: String,
    pub default_ref: String,
}
