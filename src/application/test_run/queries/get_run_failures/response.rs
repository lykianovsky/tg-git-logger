use crate::domain::test_run::entities::test_failure::TestFailure;
use crate::domain::test_run::entities::test_failure_card::TestFailureCard;

/// Упавший тест вместе с карточкой трекера, если она уже заведена: по ней решается,
/// показывать кнопку заведения или ссылку на существующую
pub struct TestFailureWithCard {
    pub failure: TestFailure,
    pub card: Option<TestFailureCard>,
}

pub struct GetRunFailuresResponse {
    pub failures: Vec<TestFailureWithCard>,
}
