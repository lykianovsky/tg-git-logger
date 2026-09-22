use crate::application::test_run::commands::create_test_failure_card::command::CreateTestFailureCardCommand;
use crate::application::test_run::commands::create_test_failure_card::error::CreateTestFailureCardError;
use crate::application::test_run::commands::create_test_failure_card::response::CreateTestFailureCardResponse;
use crate::domain::repository::entities::repository::Repository;
use crate::domain::repository::entities::repository_task_tracker::RepositoryTaskTracker;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::repository::repositories::repository_task_tracker_repository::RepositoryTaskTrackerRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::task::ports::task_tracker_client::{NewTaskTrackerCard, TaskTrackerClient};
use crate::domain::test_run::entities::test_failure::TestFailure;
use crate::domain::test_run::entities::test_failure_card::NewTestFailureCard;
use crate::domain::test_run::repositories::test_failure_card_repository::TestFailureCardRepository;
use crate::domain::test_run::repositories::test_run_repository::TestRunRepository;
use std::sync::Arc;

pub struct CreateTestFailureCardExecutor {
    test_run_repo: Arc<dyn TestRunRepository>,
    card_repo: Arc<dyn TestFailureCardRepository>,
    repository_repo: Arc<dyn RepositoryRepository>,
    repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
    task_tracker_client: Arc<dyn TaskTrackerClient>,
    kaiten_base: String,
}

impl CreateTestFailureCardExecutor {
    pub fn new(
        test_run_repo: Arc<dyn TestRunRepository>,
        card_repo: Arc<dyn TestFailureCardRepository>,
        repository_repo: Arc<dyn RepositoryRepository>,
        repository_task_tracker_repo: Arc<dyn RepositoryTaskTrackerRepository>,
        task_tracker_client: Arc<dyn TaskTrackerClient>,
        kaiten_base: String,
    ) -> Self {
        Self {
            test_run_repo,
            card_repo,
            repository_repo,
            repository_task_tracker_repo,
            task_tracker_client,
            kaiten_base,
        }
    }

    fn build_card_url(&self, tracker: &RepositoryTaskTracker, card_id: u64) -> String {
        format!(
            "{}{}",
            self.kaiten_base,
            tracker.path_to_card.replace("{id}", &card_id.to_string())
        )
    }

    fn build_title(failure: &TestFailure) -> String {
        format!("E2E: {} — {}", failure.project, failure.title)
    }

    fn build_description(failure: &TestFailure, repository: &Repository) -> String {
        let error = failure.error_excerpt.as_deref().unwrap_or("—");

        format!(
            "Репозиторий: {}/{}\nПроект: {}\nФайл: {}\nТест: {}\n\nОшибка:\n{}",
            repository.owner, repository.name, failure.project, failure.file, failure.title, error,
        )
    }
}

impl CommandExecutor for CreateTestFailureCardExecutor {
    type Command = CreateTestFailureCardCommand;
    type Response = CreateTestFailureCardResponse;
    type Error = CreateTestFailureCardError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let failure = self.test_run_repo.find_failure(cmd.test_failure_id).await?;
        let run = self.test_run_repo.find_by_id(failure.test_run_id).await?;

        let existing = self
            .card_repo
            .find_by_fingerprint(run.repository_id, &failure.fingerprint)
            .await?;

        // Открытая карточка по этому тесту уже есть — показываем её вместо второй
        if let Some(card) = existing.as_ref().filter(|card| card.closed_at.is_none()) {
            return Ok(CreateTestFailureCardResponse::AlreadyExists(Box::new(
                card.clone(),
            )));
        }

        let repository = self.repository_repo.find_by_id(run.repository_id).await?;
        let tracker = self
            .repository_task_tracker_repo
            .find_by_repository_id(run.repository_id)
            .await
            .map_err(|error| {
                tracing::warn!(%error, "Task tracker is not configured");

                CreateTestFailureCardError::TrackerNotConfigured
            })?;

        // Доска и колонка берутся из настройки трекера репозитория — той же, по которой
        // карточки переезжают при мерже
        let board_id = tracker
            .board_id
            .ok_or(CreateTestFailureCardError::TrackerNotConfigured)?;

        let created = self
            .task_tracker_client
            .create_card(&NewTaskTrackerCard {
                board_id,
                column_id: tracker.qa_column_id,
                title: Self::build_title(&failure),
                description: Self::build_description(&failure, &repository),
                responsible_id: cmd.responsible_id,
                tag: cmd.tag.clone(),
            })
            .await?;

        let card = self
            .card_repo
            .upsert(&NewTestFailureCard {
                repository_id: run.repository_id,
                fingerprint: failure.fingerprint.clone(),
                project: failure.project.clone(),
                file: failure.file.clone(),
                title: failure.title.clone(),
                card_id: created.id.0,
                card_url: self.build_card_url(&tracker, created.id.0),
                // Прошлая карточка закрыта: связываем новую с ней, чтобы видеть повтор
                previous_card_id: existing.map(|card| card.card_id),
                created_by_user_id: cmd.created_by_user_id,
            })
            .await?;

        tracing::info!(
            test_failure_id = cmd.test_failure_id,
            card_id = card.card_id,
            "Test failure card created"
        );

        Ok(CreateTestFailureCardResponse::Created(Box::new(card)))
    }
}
