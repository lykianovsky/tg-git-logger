use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Как у репозитория устроен набор тестов: какой процесс CI запускать, как передать ему
/// аргументы и где в артефакте лежат итоги и отчёт. Хранится в базе, поэтому второй проект
/// подключается записью, а не правкой кода.
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub repository_id: RepositoryId,
    /// Владелец и имя репозитория в CI — берутся из привязки репозитория
    pub owner: String,
    pub name: String,
    pub workflow_file: String,
    pub default_ref: String,
    pub args_input_name: String,
    pub ref_input_name: String,
    pub tag_input_name: String,
    pub artifact_prefix: String,
    pub summary_path: String,
    pub tests_root: String,
}
