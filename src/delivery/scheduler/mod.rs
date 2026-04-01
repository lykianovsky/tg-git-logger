use crate::application::digest::commands::send_due_digests::command::SendDueDigestsCommand;
use crate::application::health_ping::commands::check_all_health_pings::command::CheckAllHealthPingsCommand;
use crate::bootstrap::executors::ApplicationBoostrapExecutors;
use crate::config::application::ApplicationConfig;
use crate::delivery::contract::ApplicationDelivery;
use crate::domain::shared::command::CommandExecutor;
use async_trait::async_trait;
use chrono::{Timelike, Utc};
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
                                tracing::info!(
                                    sent = r.sent_count,
                                    "Digest notifications sent"
                                );
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

        scheduler.start().await.expect("JobScheduler start failed");

        tracing::info!("Scheduler started");

        // Keep the scheduler alive — dropping it stops all cron jobs
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
        }
    }
}
