use crate::application::digest::commands::send_due_digests::command::SendDueDigestsCommand;
use crate::application::health_ping::commands::check_all_health_pings::command::CheckAllHealthPingsCommand;
use crate::application::notification::commands::flush_pending_notifications::command::FlushPendingNotificationsExecutorCommand;
use crate::application::notification::commands::scan_stale_pull_requests::command::ScanStalePullRequestsExecutorCommand;
use crate::application::release_plan::commands::send_call_reminders::command::SendCallRemindersExecutorCommand;
use crate::application::release_plan::commands::send_release_day_reminders::command::SendReleaseDayRemindersExecutorCommand;
use crate::application::release_plan::queries::get_upcoming_release_plans::query::GetUpcomingReleasePlansQuery;
use crate::application::repository::queries::get_all_repositories::query::GetAllRepositoriesQuery;
use crate::application::user::queries::get_all_users::query::GetAllUsersQuery;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::domain::shared::command::CommandExecutor;
use crate::infrastructure::metrics::registry::METRICS;
use async_trait::async_trait;
use chrono::{Timelike, Utc};
use chrono_tz::Europe::Moscow;
use std::error::Error;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct DeliveryScheduler {
    executors: Arc<ApplicationBoostrapExecutors>,
    config: Arc<ApplicationConfig>,
}

impl DeliveryScheduler {
    pub fn new(
        executors: Arc<ApplicationBoostrapExecutors>,
        config: Arc<ApplicationConfig>,
    ) -> Self {
        Self { executors, config }
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
                                tracing::info!(
                                    sent = r.sent_count,
                                    "Release day reminders sent"
                                );
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
                                tracing::info!(
                                    sent = r.sent_count,
                                    "Release call reminders sent"
                                );
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

        // Business metrics — каждую минуту на :45 секунде
        let metrics_executors = self.executors.clone();
        scheduler
            .add(
                Job::new_async("45 * * * * *", move |_uuid, _lock| {
                    let executors = metrics_executors.clone();
                    Box::pin(async move {
                        let today_msk = Utc::now().with_timezone(&Moscow).date_naive();

                        if let Ok(r) = executors
                            .queries
                            .get_upcoming_release_plans
                            .execute(&GetUpcomingReleasePlansQuery {
                                from_date: today_msk,
                            })
                            .await
                        {
                            METRICS
                                .release_plans_active_total
                                .set(r.plans.len() as i64);
                            let today_count = r
                                .plans
                                .iter()
                                .filter(|p| p.planned_date == today_msk)
                                .count();
                            METRICS
                                .release_plans_today_total
                                .set(today_count as i64);
                        }

                        if let Ok(r) = executors
                            .queries
                            .get_all_repositories
                            .execute(&GetAllRepositoriesQuery {})
                            .await
                        {
                            METRICS
                                .repositories_total
                                .set(r.repositories.len() as i64);
                        }

                        if let Ok(r) = executors
                            .queries
                            .get_all_users
                            .execute(&GetAllUsersQuery {})
                            .await
                        {
                            let active_count =
                                r.users.iter().filter(|u| u.is_active).count() as i64;
                            let inactive_count = r.users.len() as i64 - active_count;
                            METRICS
                                .users_total
                                .with_label_values(&["active"])
                                .set(active_count);
                            METRICS
                                .users_total
                                .with_label_values(&["inactive"])
                                .set(inactive_count);
                        }

                        // users_on_vacation_total: TODO query user_preferences where vacation_until > now
                        METRICS.users_on_vacation_total.set(0);
                    })
                })
                .expect("Business metrics job create error"),
            )
            .await
            .expect("JobScheduler failed to add business metrics job");

        scheduler.start().await.expect("JobScheduler start failed");

        tracing::info!("Scheduler started");

        // Keep the scheduler alive — dropping it stops all cron jobs
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
