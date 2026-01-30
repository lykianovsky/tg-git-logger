use crate::config::environment::ENV;
use crate::domain::notification::service::NotificationService;
use crate::domain::task_tracker::service::TaskTrackerService;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;

pub struct NotifyReceivedWebhookUseCase {
    notifier: Arc<dyn NotificationService>,
    task_tracker: Arc<dyn TaskTrackerService>
}

impl NotifyReceivedWebhookUseCase {
    pub fn new(
        notifier: Arc<dyn NotificationService>,
        task_tracker: Arc<dyn TaskTrackerService>
    ) -> Self {
        Self {
            notifier,
            task_tracker
        }
    }

    pub fn execute(&self, text: &MessageBuilder) {
        let notifier = Arc::clone(&self.notifier);
        
        let text = MessageBuilder::new().raw(
            &self.task_tracker.linkify_tasks_in_text(
                text.clone().to_string().as_str()
            )
        );

        tokio::spawn(async move {
            let chat_id: i64 = ENV.get("TELEGRAM_CHAT_ID").parse().unwrap();
            let _ = notifier.send_to_chat(chat_id, &text).await;
        });
    }
}