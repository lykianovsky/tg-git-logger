use crate::domain::test_run::entities::test_run::TestRun;

pub struct IngestTestRunResultResponse {
    pub run: TestRun,
    /// Итогов в артефакте не было: показываем статус из CI и ссылку на прогон
    pub summary_missing: bool,
}
