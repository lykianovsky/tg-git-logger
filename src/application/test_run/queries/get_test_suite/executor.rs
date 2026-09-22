use crate::application::test_run::queries::get_test_suite::error::GetTestSuiteError;
use crate::application::test_run::queries::get_test_suite::query::GetTestSuiteQuery;
use crate::application::test_run::queries::get_test_suite::response::GetTestSuiteResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::test_run::repositories::test_suite_repository::{
    FindTestSuiteError, TestSuiteRepository,
};
use std::sync::Arc;

pub struct GetTestSuiteExecutor {
    test_suite_repo: Arc<dyn TestSuiteRepository>,
}

impl GetTestSuiteExecutor {
    pub fn new(test_suite_repo: Arc<dyn TestSuiteRepository>) -> Self {
        Self { test_suite_repo }
    }
}

impl CommandExecutor for GetTestSuiteExecutor {
    type Command = GetTestSuiteQuery;
    type Response = GetTestSuiteResponse;
    type Error = GetTestSuiteError;

    async fn execute(&self, query: &Self::Command) -> Result<Self::Response, Self::Error> {
        match self
            .test_suite_repo
            .find_by_repository(query.repository_id)
            .await
        {
            Ok(suite) => Ok(GetTestSuiteResponse { suite: Some(suite) }),
            // Не подключены — это обычное состояние, а не ошибка
            Err(FindTestSuiteError::NotConfigured) => Ok(GetTestSuiteResponse { suite: None }),
            Err(FindTestSuiteError::DbError(message)) => Err(GetTestSuiteError::DbError(message)),
        }
    }
}
