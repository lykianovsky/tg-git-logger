use crate::config::environment::ENV;
use crate::domain::notification::service::NotificationService;
use crate::domain::task_tracker::service::TaskTrackerService;
use crate::domain::user::repository::UserRepository;
use crate::infrastructure::repository::mysql::user::MySQLUserRepository;
use crate::infrastructure::services::kaiten::KaitenTaskTrackerService;
use crate::infrastructure::services::telegram::notifier::TelegramNotifierService;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct ApplicationState {
    pub db: Arc<DatabaseConnection>,
    pub services: ApplicationServices,
    pub repositories: ApplicationRepositories
}

pub struct ApplicationServices {
    pub notifier: Arc<dyn NotificationService>,
    pub task_tracker: Arc<dyn TaskTrackerService>,
}

pub struct ApplicationRepositories {
    pub user: Arc<dyn UserRepository + Send + Sync>
}

impl ApplicationState {
    pub fn new(db: DatabaseConnection) -> Self {
        let db = Arc::new(db);

        let services = Self::build_services();
        let repositories = Self::build_repositories(Arc::clone(&db));

        Self { db, services, repositories }
    }

    fn build_services() -> ApplicationServices {
        let task_tracker: Arc<dyn TaskTrackerService> = Arc::new(
            KaitenTaskTrackerService::new()
        );

        let notifier: Arc<dyn NotificationService> = Arc::new(
            TelegramNotifierService::new(
                ENV.get("TELEGRAM_BOT_TOKEN")
            )
        );

        ApplicationServices {
            notifier,
            task_tracker
        }
    }

    fn build_repositories(db: Arc<DatabaseConnection>) -> ApplicationRepositories {
        let user_repository: Arc<dyn UserRepository> = Arc::new(
            MySQLUserRepository::new(db)
        );

        ApplicationRepositories {
            user: user_repository
        }
    }
}