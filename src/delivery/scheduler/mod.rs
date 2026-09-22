use crate::application::digest::commands::send_due_digests::command::SendDueDigestsCommand;
use crate::application::health_ping::commands::check_all_health_pings::command::CheckAllHealthPingsCommand;
use crate::application::notification::commands::flush_pending_notifications::command::FlushPendingNotificationsExecutorCommand;
use crate::application::notification::commands::scan_pr_conflicts::command::ScanPrConflictsExecutorCommand;
use crate::application::notification::commands::scan_stale_pull_requests::command::ScanStalePullRequestsExecutorCommand;
use crate::application::release_plan::commands::send_call_reminders::command::SendCallRemindersExecutorCommand;
use crate::application::release_plan::commands::send_release_day_reminders::command::SendReleaseDayRemindersExecutorCommand;
use crate::application::test_run::commands::sync_stale_test_runs::command::SyncStaleTestRunsCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::bootstrap::shared_dependency::ApplicationSharedDependency;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::delivery::notifications::test_run::{
    build_test_run_card, deliver_test_run_update, resolve_test_run_chat_id,
};
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use async_trait::async_trait;
use chrono::{Timelike, Utc};
use std::error::Error;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct DeliveryScheduler {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
    shared_dependency: Arc<ApplicationSharedDependency>,
}

impl DeliveryScheduler {
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
impl ApplicationDelivery for DeliveryScheduler {
    async fn serve(&self) -> Result<(), Box<dyn Error>> {
        let scheduler = JobScheduler::new()
            .await
            .expect("JobScheduler not initialized");

        // Health ping checker — every minute
        let executors = self.executors.clone();

        scheduler
            .add(
                Job::new_async("0 * * * * *", move |_uuid, _lock| {
                    let executors = executors.clone();

                    Box::pin(async move {
                        match executors
                            .commands
                            .check_all_health_pings
                            .execute(&CheckAllHealthPingsCommand)
                            .await
                        {
                            Ok(r) if r.checked_count > 0 => {
                                tracing::debug!(
                                    checked = r.checked_count,
                                    failed = r.failed_count,
                                    recovered = r.recovered_count,
                                    "Health ping check completed"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    "Health ping check failed"
                                );
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Health ping job create error"),
            )
            .await
            .expect("JobScheduler failed to add health ping job");

        // Digest sender — every minute at :30 (staggered from health pings)
        let digest_executors = self.executors.clone();

        scheduler
            .add(
                Job::new_async("30 * * * * *", move |_uuid, _lock| {
                    let executors = digest_executors.clone();

                    Box::pin(async move {
                        let now = Utc::now();
                        let cmd = SendDueDigestsCommand {
                            hour: now.hour() as i8,
                            minute: now.minute() as i8,
                        };

                        match executors.commands.send_due_digests.execute(&cmd).await {
                            Ok(r) if r.sent_count > 0 => {
                                tracing::info!(sent = r.sent_count, "Digest notifications sent");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Digest send failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Digest job create error"),
            )
            .await
            .expect("JobScheduler failed to add digest job");

        // Pending notifications flush — every minute at :15 (staggered)
        let flush_executors = self.executors.clone();

        scheduler
            .add(
                Job::new_async("15 * * * * *", move |_uuid, _lock| {
                    let executors = flush_executors.clone();

                    Box::pin(async move {
                        match executors
                            .commands
                            .flush_pending_notifications
                            .execute(&FlushPendingNotificationsExecutorCommand {})
                            .await
                        {
                            Ok(r) if r.flushed_count > 0 => {
                                tracing::info!(
                                    flushed = r.flushed_count,
                                    "Pending notifications flushed"
                                );
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Pending notifications flush failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Pending flush job create error"),
            )
            .await
            .expect("JobScheduler failed to add pending flush job");

        // Stale PR digest — раз в день в 15:00 МСК (12:00 UTC)
        let stale_executors = self.executors.clone();
        scheduler
            .add(
                Job::new_async("0 0 12 * * *", move |_uuid, _lock| {
                    let executors = stale_executors.clone();
                    Box::pin(async move {
                        match executors
                            .commands
                            .scan_stale_pull_requests
                            .execute(&ScanStalePullRequestsExecutorCommand {})
                            .await
                        {
                            Ok(r) if r.stale_count > 0 => {
                                tracing::info!(
                                    repos = r.repos_scanned,
                                    stale = r.stale_count,
                                    "Stale PR digest sent"
                                );
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Stale PR scan failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Stale PR scan job create error"),
            )
            .await
            .expect("JobScheduler failed to add stale PR job");

        // PR conflict scan — каждые 30 минут
        let conflict_executors = self.executors.clone();
        scheduler
            .add(
                Job::new_async("0 */30 * * * *", move |_uuid, _lock| {
                    let executors = conflict_executors.clone();
                    Box::pin(async move {
                        match executors
                            .commands
                            .scan_pr_conflicts
                            .execute(&ScanPrConflictsExecutorCommand)
                            .await
                        {
                            Ok(r) if r.conflicts_count > 0 => {
                                tracing::info!(
                                    repos = r.repos_scanned,
                                    conflicts = r.conflicts_count,
                                    "PR conflict notifications sent"
                                );
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "PR conflict scan failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("PR conflict scan job create error"),
            )
            .await
            .expect("JobScheduler failed to add PR conflict scan job");

        // Release day reminder — каждый день в 10:00 МСК (07:00 UTC)
        let release_day_executors = self.executors.clone();
        scheduler
            .add(
                Job::new_async("0 0 7 * * *", move |_uuid, _lock| {
                    let executors = release_day_executors.clone();
                    Box::pin(async move {
                        match executors
                            .commands
                            .send_release_day_reminders
                            .execute(&SendReleaseDayRemindersExecutorCommand)
                            .await
                        {
                            Ok(r) if r.sent_count > 0 => {
                                tracing::info!(sent = r.sent_count, "Release day reminders sent");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Release day reminders failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Release day reminder job create error"),
            )
            .await
            .expect("JobScheduler failed to add release day reminder job");

        // Call reminder — каждые 15 минут (в т.ч. в :00, :15, :30, :45)
        let call_reminder_executors = self.executors.clone();
        scheduler
            .add(
                Job::new_async("0 */15 * * * *", move |_uuid, _lock| {
                    let executors = call_reminder_executors.clone();
                    Box::pin(async move {
                        match executors
                            .commands
                            .send_call_reminders
                            .execute(&SendCallRemindersExecutorCommand)
                            .await
                        {
                            Ok(r) if r.sent_count > 0 => {
                                tracing::info!(sent = r.sent_count, "Release call reminders sent");
                            }
                            Err(e) => {
                                tracing::error!(error = %e, "Release call reminders failed");
                            }
                            _ => {}
                        }
                    })
                })
                .expect("Call reminder job create error"),
            )
            .await
            .expect("JobScheduler failed to add call reminder job");

        scheduler.start().await.expect("JobScheduler start failed");

        tracing::info!("Scheduler started");

        // Итоги прогонов тестов — подстраховка, если вебхук не дошёл
        let test_runs_executors = self.executors.clone();
        let test_runs_shared = self.shared_dependency.clone();
        let test_runs_default_chat_id = SocialChatId(self.config.telegram.chat_id);

        scheduler
            .add(
                Job::new_async("*/20 * * * * *", move |_uuid, _lock| {
                    let executors = test_runs_executors.clone();
                    let shared_dependency = test_runs_shared.clone();

                    Box::pin(async move {
                        let notification_service: Arc<dyn NotificationService> =
                            shared_dependency.notification_service.clone();

                        let response = match executors
                            .commands
                            .sync_stale_test_runs
                            .execute(&SyncStaleTestRunsCommand {})
                            .await
                        {
                            Ok(response) => response,
                            Err(error) => {
                                tracing::error!(error = %error, "Stale test runs sync failed");

                                return;
                            }
                        };

                        // Карточку держим живой: пока прогон идёт, обновляем её сами
                        for run in response.active.iter().chain(response.finished.iter()) {
                            let chat_id = resolve_test_run_chat_id(
                                &shared_dependency.repository_repo,
                                run,
                                test_runs_default_chat_id,
                            )
                            .await;
                            let (message, keyboard) = build_test_run_card(
                                &shared_dependency.repository_repo,
                                &executors.queries.get_test_run_progress,
                                run,
                            )
                            .await;

                            deliver_test_run_update(
                                &notification_service,
                                &shared_dependency.publisher,
                                run,
                                chat_id,
                                message,
                                Some(keyboard),
                            )
                            .await;
                        }
                    })
                })
                .expect("Stale test runs job create error"),
            )
            .await
            .expect("JobScheduler failed to add stale test runs job");

        // Keep the scheduler alive — dropping it stops all cron jobs
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
