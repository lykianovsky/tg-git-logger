pub mod github;
pub mod user;

use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::events::listeners::github::webhook::pull_request::WebhookPullRequestEventListener;
use crate::delivery::events::listeners::github::webhook::pull_request_review::WebhookPullRequestReviewEventListener;
use crate::delivery::events::listeners::github::webhook::push::WebhookPushEventListener;
use crate::delivery::events::listeners::github::webhook::release::WebhookReleaseEventListener;
use crate::delivery::events::listeners::github::webhook::workflow::WebhookWorkflowEventListener;
use crate::delivery::events::listeners::user::registration::failed::UserRegistrationFailedListener;
use crate::delivery::events::listeners::user::registration::success::UserRegistrationSuccessListener;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use async_trait::async_trait;
use std::error::Error;
use std::sync::Arc;

pub struct DeliveryEventListeners {
    shared_dependency: Arc<ApplicationSharedDependency>,
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryEventListeners {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
        shared_dependency: Arc<ApplicationSharedDependency>,
    ) -> Self {
        Self {
            executors,
            config,
            shared_dependency,
        }
    }
}

#[async_trait]
impl ApplicationDelivery for DeliveryEventListeners {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let default_chat_id = SocialChatId(self.config.telegram.chat_id);
        let repository_repo = self.shared_dependency.repository_repo.clone();

        // Version Control Webhooks
        self.shared_dependency
            .event_bus
            .on(WebhookPullRequestEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                repository_repo: repository_repo.clone(),
                repository_task_tracker_repo: self
                    .shared_dependency
                    .repository_task_tracker_repo
                    .clone(),
                default_chat_id,
                task_tracker_service: self.shared_dependency.task_tracker_service.clone(),
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookPushEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                repository_repo: repository_repo.clone(),
                default_chat_id,
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookReleaseEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                repository_repo: repository_repo.clone(),
                default_chat_id,
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(WebhookWorkflowEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                repository_repo: repository_repo.clone(),
                default_chat_id,
            })
            .await;

        // PR review DM notifications (approved / changes_requested)
        self.shared_dependency
            .event_bus
            .on(WebhookPullRequestReviewEventListener {
                publisher: self.shared_dependency.publisher.clone(),
                user_vc_accounts_repo: self.shared_dependency.user_version_controls_repo.clone(),
                user_socials_repo: self.shared_dependency.user_socials_repo.clone(),
            })
            .await;

        // PR comment DM notifications
        // self.shared_dependency
        //     .event_bus
        //     .on(WebhookPrCommentEventListener {
        //         publisher: self.shared_dependency.publisher.clone(),
        //         user_vc_accounts_repo: self.shared_dependency.user_version_controls_repo.clone(),
        //         user_socials_repo: self.shared_dependency.user_socials_repo.clone(),
        //     })
        //     .await;

        // UserRegistration
        self.shared_dependency
            .event_bus
            .on(UserRegistrationSuccessListener {
                publisher: self.shared_dependency.publisher.clone(),
                telegram_admin_user_id: SocialUserId(self.config.telegram.admin_user_id as i32),
            })
            .await;
        self.shared_dependency
            .event_bus
            .on(UserRegistrationFailedListener {
                publisher: self.shared_dependency.publisher.clone(),
            })
            .await;

        Ok(())
    }
}
