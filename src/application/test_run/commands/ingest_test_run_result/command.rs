use crate::domain::test_run::value_objects::run_tag::RunTag;

/// Прогон завершился в CI: метку берём из имени прогона в вебхуке
pub struct IngestTestRunResultCommand {
    pub run_tag: RunTag,
}
