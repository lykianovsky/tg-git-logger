use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::test_run::entities::test_failure_card::{NewTestFailureCard, TestFailureCard};
use crate::domain::test_run::repositories::test_failure_card_repository::{
    FindTestFailureCardError, SaveTestFailureCardError, TestFailureCardRepository,
};
use crate::domain::test_run::value_objects::test_fingerprint::TestFingerprint;
use crate::domain::user::value_objects::user_id::UserId;
use crate::infrastructure::database::mysql::entities::test_failure_cards;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct MySQLTestFailureCardRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLTestFailureCardRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    fn from_mysql(model: test_failure_cards::Model) -> TestFailureCard {
        TestFailureCard {
            id: model.id,
            repository_id: RepositoryId(model.repository_id),
            fingerprint: TestFingerprint(model.fingerprint),
            project: model.project,
            file: model.file,
            title: model.title,
            card_id: model.card_id.max(0) as u64,
            card_url: model.card_url,
            previous_card_id: model.previous_card_id.map(|id| id.max(0) as u64),
            created_by_user_id: model.created_by_user_id.map(UserId),
            created_at: model.created_at,
            closed_at: model.closed_at,
        }
    }
}

#[async_trait]
impl TestFailureCardRepository for MySQLTestFailureCardRepository {
    async fn find_by_fingerprint(
        &self,
        repository_id: RepositoryId,
        fingerprint: &TestFingerprint,
    ) -> Result<Option<TestFailureCard>, FindTestFailureCardError> {
        let model = test_failure_cards::Entity::find()
            .filter(test_failure_cards::Column::RepositoryId.eq(repository_id.0))
            .filter(test_failure_cards::Column::Fingerprint.eq(fingerprint.as_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|error| FindTestFailureCardError::DbError(error.to_string()))?;

        Ok(model.map(Self::from_mysql))
    }

    async fn upsert(
        &self,
        card: &NewTestFailureCard,
    ) -> Result<TestFailureCard, SaveTestFailureCardError> {
        let existing = test_failure_cards::Entity::find()
            .filter(test_failure_cards::Column::RepositoryId.eq(card.repository_id.0))
            .filter(test_failure_cards::Column::Fingerprint.eq(card.fingerprint.as_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|error| SaveTestFailureCardError::DbError(error.to_string()))?;

        // Карточка на тест одна: запись существует, когда прошлая карточка закрыта
        // и по тому же тесту заводится новая
        let model = match existing {
            Some(existing) => test_failure_cards::ActiveModel {
                id: Set(existing.id),
                card_id: Set(card.card_id as i64),
                card_url: Set(card.card_url.clone()),
                previous_card_id: Set(card.previous_card_id.map(|id| id as i64)),
                created_by_user_id: Set(card.created_by_user_id.map(|id| id.0)),
                created_at: Set(Utc::now()),
                closed_at: Set(None),
                ..Default::default()
            }
            .update(self.db.as_ref())
            .await
            .map_err(|error| SaveTestFailureCardError::DbError(error.to_string()))?,
            None => test_failure_cards::ActiveModel {
                repository_id: Set(card.repository_id.0),
                fingerprint: Set(card.fingerprint.0.clone()),
                project: Set(card.project.clone()),
                file: Set(card.file.clone()),
                title: Set(card.title.clone()),
                card_id: Set(card.card_id as i64),
                card_url: Set(card.card_url.clone()),
                previous_card_id: Set(card.previous_card_id.map(|id| id as i64)),
                created_by_user_id: Set(card.created_by_user_id.map(|id| id.0)),
                ..Default::default()
            }
            .insert(self.db.as_ref())
            .await
            .map_err(|error| SaveTestFailureCardError::DbError(error.to_string()))?,
        };

        Ok(Self::from_mysql(model))
    }

    async fn mark_closed(
        &self,
        id: i32,
        closed_at: DateTime<Utc>,
    ) -> Result<(), SaveTestFailureCardError> {
        let model = test_failure_cards::ActiveModel {
            id: Set(id),
            closed_at: Set(Some(closed_at)),
            ..Default::default()
        };

        model
            .update(self.db.as_ref())
            .await
            .map_err(|error| SaveTestFailureCardError::DbError(error.to_string()))?;

        Ok(())
    }
}
