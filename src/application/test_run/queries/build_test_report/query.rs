use crate::domain::test_run::value_objects::test_run_id::TestRunId;

pub struct BuildTestReportQuery {
    pub test_run_id: TestRunId,
}
