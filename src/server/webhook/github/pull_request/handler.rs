use crate::config::environment::ENV;
use crate::server::notifier::NotifierService;
use crate::server::task_tracker::TaskTrackerService;
use crate::server::webhook::github::pull_request::payload::PullRequestEvent;
use crate::utils::notifier::message_builder::MessageBuilder;
use std::sync::Arc;

pub struct PullRequestHandler {
    task_tracker: Arc<TaskTrackerService>,
    notifier: Arc<NotifierService>,
}

impl PullRequestHandler {
    pub fn new(task_tracker: Arc<TaskTrackerService>, notifier: Arc<NotifierService>) -> Self {
        Self {
            task_tracker,
            notifier,
        }
    }

    pub fn handle(&self, event: &PullRequestEvent) {
        match event.action.as_str() {
            "opened" => self.on_opened(event),
            "closed" => {
                if event.pull_request.merged {
                    self.on_merged(event);
                } else {
                    self.on_closed(event);
                }
            }
            "reopened" => self.on_reopened(event),
            "synchronize" => self.on_synchronize(event),
            _ => tracing::info!("Unhandled pull request action: {}", event.action),
        }
    }

    fn on_opened(&self, event: &PullRequestEvent) {
        tracing::info!("PR #{} opened", event.number);
    }

    fn on_closed(&self, event: &PullRequestEvent) {
        tracing::info!("PR #{} closed but not merged", event.number);
    }

    fn on_reopened(&self, event: &PullRequestEvent) {
        tracing::info!("PR #{} reopened", event.number);
    }

    fn on_synchronize(&self, event: &PullRequestEvent) {
        tracing::info!("PR #{} updated", event.number);
    }

    fn on_merged(&self, event: &PullRequestEvent) {
        let client = Arc::clone(&self.task_tracker.client);
        let notifier = Arc::clone(&self.notifier);

        // Копируем только нужные данные
        let title = event.pull_request.title.clone();
        let pr_number = event.number;
        let card_id = TaskTrackerService::extract_id(&title).unwrap_or_default();
        let pr_url = event.pull_request.html_url.clone();
        let repo_name = event.repository.full_name.clone();
        let repo_url = event.repository.html_url.clone().unwrap_or_default();

        if card_id.is_empty() {
            tracing::warn!("Task not found in PR title #{}: {}", pr_number, title);
            return;
        }

        tokio::spawn(async move {
            match client
                .move_card(
                    card_id.as_str(),
                    ENV.get("TASK_TRACKER_QA_COLUMN_ID").as_str(),
                )
                .await
            {
                Ok(_) => {
                    let message = MessageBuilder::new()
                        .bold("📌 Карточка перемещена")
                        .line(&format!(
                            "Карточка {} связанная с PR <a href=\"{}\">#{}</a> была перемещена в колонку QA.",
                            TaskTrackerService::link_by_id(&card_id, &format!("#{}", &card_id)),
                            repo_url,
                            pr_number
                        ))
                        .line(&format!("PR: <a href=\"{}\">{}</a>", pr_url, title))
                        .line(&format!(
                            "Репозиторий: <a href=\"{}\">{}</a>",
                            repo_url, repo_name
                        ));

                    notifier.notify_async(Arc::new(message));

                    tracing::info!(
                        "Task with id {} from PR #{} moved to QA column",
                        card_id,
                        pr_number
                    );
                }
                Err(e) => tracing::error!(error = %e, "Failed to move card in TaskTracker"),
            }
        });
    }
}
