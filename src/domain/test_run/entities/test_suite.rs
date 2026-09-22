use crate::domain::repository::value_objects::repository_id::RepositoryId;

/// Как у репозитория устроен набор тестов: какой процесс CI запускать, как передать ему
/// аргументы и где в артефакте лежат итоги и отчёт. Хранится в базе, поэтому второй проект
/// подключается записью, а не правкой кода.
/// Значения по умолчанию повторяют наш workflow e2e: при подключении репозитория
/// человека спрашиваем только про файл процесса и ветку
pub const DEFAULT_ARGS_INPUT_NAME: &str = "e2e";
pub const DEFAULT_REF_INPUT_NAME: &str = "ref";
pub const DEFAULT_TAG_INPUT_NAME: &str = "run_tag";
pub const DEFAULT_ARTIFACT_PREFIX: &str = "e2e-report";
pub const DEFAULT_SUMMARY_PATH: &str = "test-results/summary.json";
pub const DEFAULT_TESTS_ROOT: &str = "e2e/tests";

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

/// Подключение набора тестов: остальные поля берутся из значений по умолчанию
#[derive(Debug, Clone)]
pub struct NewTestSuite {
    pub repository_id: RepositoryId,
    pub workflow_file: String,
    pub default_ref: String,
}
