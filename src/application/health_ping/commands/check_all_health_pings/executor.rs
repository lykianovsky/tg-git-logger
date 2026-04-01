use crate::application::health_ping::commands::check_all_health_pings::command::CheckAllHealthPingsCommand;
use crate::application::health_ping::commands::check_all_health_pings::error::CheckAllHealthPingsExecutorError;
use crate::application::health_ping::commands::check_all_health_pings::response::CheckAllHealthPingsResponse;
use crate::domain::health_ping::ports::health_check_client::HealthCheckClient;
use crate::domain::health_ping::repositories::health_ping_repository::HealthPingRepository;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use std::sync::Arc;

pub struct CheckAllHealthPingsExecutor {
    health_ping_repo: Arc<dyn HealthPingRepository>,
    health_check_client: Arc<dyn HealthCheckClient>,
    notification_service: Arc<dyn NotificationService>,
    user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
}

impl CheckAllHealthPingsExecutor {
    pub fn new(
        health_ping_repo: Arc<dyn HealthPingRepository>,
        health_check_client: Arc<dyn HealthCheckClient>,
        notification_service: Arc<dyn NotificationService>,
        user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    ) -> Self {
        Self {
            health_ping_repo,
            health_check_client,
            notification_service,
            user_has_roles_repo,
            user_socials_repo,
        }
    }

    async fn notify_admins(&self, message: &MessageBuilder) {
        let admin_user_ids = match self
            .user_has_roles_repo
            .find_user_ids_by_role(RoleName::Admin)
            .await
        {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "Failed to fetch admin user IDs for health ping notification"
                );
                return;
            }
        };

        for user_id in &admin_user_ids {
            let social = match self.user_socials_repo.find_by_user_id(user_id).await
            {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        user_id = user_id.0,
                        "Failed to find social account for admin"
                    );
                    continue;
                }
            };

            let chat_id = SocialChatId(social.social_user_id.0 as i64);

            if let Err(e) = self
                .notification_service
                .send_message(&SocialType::Telegram, &chat_id, message)
                .await
            {
                tracing::error!(
                    error = %e,
                    user_id = user_id.0,
                    "Failed to send health ping notification to admin"
                );
            }
        }
    }
}

impl CommandExecutor for CheckAllHealthPingsExecutor {
    type Command = CheckAllHealthPingsCommand;
    type Response = CheckAllHealthPingsResponse;
    type Error = CheckAllHealthPingsExecutorError;

    async fn execute(
        &self,
        _cmd: &Self::Command,
    ) -> Result<Self::Response, Self::Error> {
        let pings = self.health_ping_repo.find_active_due().await?;

        let now = Utc::now();
        let mut checked_count = 0;
        let mut failed_count = 0;
        let mut recovered_count = 0;

        for ping in &pings {
            let result = self.health_check_client.check(&ping.url).await;

            let mut updated_ping = ping.clone();
            updated_ping.last_status = Some(result.status_text.clone());
            updated_ping.last_response_ms = Some(result.response_ms);
            updated_ping.last_error_message = result.error_message.clone();
            updated_ping.last_checked_at = Some(now);

            let was_ok = ping.last_status.as_deref() != Some("error");

            if result.is_success {
                // Recovery: was failing, now ok
                if let Some(failed_since) = ping.failed_since {
                    let downtime = now.signed_duration_since(failed_since);
                    let downtime_text = format_duration(downtime);

                    updated_ping.failed_since = None;

                    let message = MessageBuilder::new()
                        .with_html_escape(true)
                        .bold("✅ Сервис восстановлен")
                        .empty_line()
                        .section("Сервис", &ping.name)
                        .section("URL", &ping.url)
                        .section(
                            "Время ответа",
                            &format!("{} мс", result.response_ms),
                        )
                        .section("Был недоступен", &downtime_text);

                    self.notify_admins(&message).await;

                    recovered_count += 1;

                    tracing::info!(
                        ping_name = %ping.name,
                        downtime = %downtime_text,
                        response_ms = result.response_ms,
                        "Health ping recovered"
                    );
                }
            } else {
                // Failure
                if was_ok || ping.failed_since.is_none() {
                    // First failure — set failed_since, send notification
                    updated_ping.failed_since = Some(now);

                    let error_text = result
                        .error_message
                        .as_deref()
                        .unwrap_or("unknown");

                    let message = MessageBuilder::new()
                        .with_html_escape(true)
                        .bold("🔴 Сервис недоступен")
                        .empty_line()
                        .section("Сервис", &ping.name)
                        .section("URL", &ping.url)
                        .section("Ошибка", error_text);

                    self.notify_admins(&message).await;

                    tracing::warn!(
                        ping_name = %ping.name,
                        error = error_text,
                        "Health ping failed"
                    );
                }
                // Repeated failure — failed_since already set, don't spam

                failed_count += 1;
            }

            if let Err(e) = self.health_ping_repo.update(&updated_ping).await {
                tracing::error!(
                    error = %e,
                    ping_name = %ping.name,
                    "Failed to update health ping status"
                );
            }

            checked_count += 1;
        }

        Ok(CheckAllHealthPingsResponse {
            checked_count,
            failed_count,
            recovered_count,
        })
    }
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 60 {
        return format!("{} сек", total_seconds);
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if hours > 0 {
        format!("{} ч {} мин", hours, minutes)
    } else {
        format!("{} мин", minutes)
    }
}
