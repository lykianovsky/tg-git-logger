use crate::application::digest::commands::send_due_digests::command::SendDueDigestsCommand;
use crate::application::health_ping::commands::update_health_ping_status::command::UpdateHealthPingStatusCommand;
use crate::application::health_ping::queries::get_all_health_pings::query::GetAllHealthPingsQuery;
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
                        if let Err(e) = check_health_pings(&executors).await {
                            tracing::error!(error = %e, "Health ping check failed");
                        }
                    })
                })
                .expect("Health ping job create error"),
            )
            .await
            .expect("JobScheduler failed to add health ping job");

        // Digest sender — every minute
        let digest_executors = self.executors.clone();

        scheduler
            .add(
                Job::new_async("0 * * * * *", move |_uuid, _lock| {
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

        Ok(())
    }
}

async fn check_health_pings(
    executors: &ApplicationBoostrapExecutors,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pings = executors
        .queries
        .get_all_health_pings
        .execute(&GetAllHealthPingsQuery)
        .await?;

    let now = Utc::now();

    for ping in &pings.pings {
        if !ping.is_active {
            continue;
        }

        let is_due = match ping.last_checked_at {
            None => true,
            Some(last) => {
                let elapsed = now.signed_duration_since(last);
                elapsed.num_minutes() >= ping.interval_minutes as i64
            }
        };

        if !is_due {
            continue;
        }

        let start = std::time::Instant::now();

        let result = reqwest::get(&ping.url).await;

        let elapsed_ms = start.elapsed().as_millis() as i32;

        let (status, error_message) = match result {
            Ok(response) => {
                if response.status().is_success() {
                    ("ok".to_string(), None)
                } else {
                    (
                        "error".to_string(),
                        Some(format!("HTTP {}", response.status())),
                    )
                }
            }

            Err(e) => ("error".to_string(), Some(e.to_string())),
        };

        let cmd = UpdateHealthPingStatusCommand {
            id: ping.id,
            status: status.clone(),
            response_ms: Some(elapsed_ms),
            error_message: error_message.clone(),
            checked_at: Utc::now(),
        };

        if let Err(e) = executors
            .commands
            .update_health_ping_status
            .execute(&cmd)
            .await
        {
            tracing::error!(
                error = %e,
                ping_name = %ping.name,
                "Failed to update health ping status"
            );
        }

        // Log status changes
        let previous = ping.last_status.as_deref();
        let new = status.as_str();

        if previous != Some(new) {
            match new {
                "ok" => {
                    tracing::info!(
                        ping_name = %ping.name,
                        response_ms = elapsed_ms,
                        "Health ping recovered"
                    );
                }

                "error" => {
                    tracing::warn!(
                        ping_name = %ping.name,
                        error = error_message.as_deref().unwrap_or("unknown"),
                        "Health ping failed"
                    );
                }

                _ => {}
            }
        }
    }

    Ok(())
}
