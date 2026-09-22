use crate::domain::test_run::entities::test_failure_card::TestFailureCard;

pub enum CreateTestFailureCardResponse {
    /// Карточка по этому тесту уже заведена и ещё не закрыта — новую не создаём
    AlreadyExists(Box<TestFailureCard>),
    Created(Box<TestFailureCard>),
}
