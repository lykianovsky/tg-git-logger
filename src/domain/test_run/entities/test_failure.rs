use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use crate::domain::test_run::value_objects::test_run_id::TestRunId;

#[derive(Debug, Clone)]
pub struct TestFailure {
    pub id: i32,
    pub test_run_id: TestRunId,
    pub project: String,
    pub file: String,
    pub title: String,
    pub fingerprint: TestFingerprint,
    pub error_excerpt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewTestFailure {
    pub project: String,
    pub file: String,
    pub title: String,
    pub fingerprint: TestFingerprint,
    pub error_excerpt: Option<String>,
}
