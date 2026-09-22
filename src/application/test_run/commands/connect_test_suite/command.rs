use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Подключение репозитория к тестам: процесс CI и ветка по умолчанию.
/// Остальное берётся из значений по умолчанию, а место для карточек — из настроек трекера
pub struct ConnectTestSuiteCommand {
    pub repository_id: RepositoryId,
    pub workflow_file: String,
    pub default_ref: String,
}
