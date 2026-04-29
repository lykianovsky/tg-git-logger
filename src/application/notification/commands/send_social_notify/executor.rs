use crate::application::notification::commands::buffer_notification::command::BufferNotificationExecutorCommand;
use crate::application::notification::commands::buffer_notification::executor::BufferNotificationExecutor;
use crate::application::notification::commands::send_social_notify::command::SendSocialNotifyExecutorCommand;
use crate::application::notification::commands::send_social_notify::error::SendSocialNotifyExecutorError;
use crate::application::notification::commands::send_social_notify::response::SendSocialNotifyExecutorResponse;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user_preferences::repositories::user_preferences_repository::UserPreferencesRepository;
use crate::domain::user_preferences::services::quiet_hours_resolver::QuietHoursResolver;
use chrono::{Duration, Utc};
use std::sync::Arc;

const DEFAULT_EVENT_TYPE: &str = "general";
// Уведомления, попадающие на отпуск длиннее этого порога, дропаются без буферизации,
// чтобы избежать лавины из сотен сообщений в момент возврата.
const VACATION_DROP_THRESHOLD_HOURS: i64 = 48;

pub struct SendSocialNotifyExecutor {
    notification_service: Arc<dyn NotificationService>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_preferences_repo: Arc<dyn UserPreferencesRepository>,
    quiet_hours_resolver: Arc<QuietHoursResolver>,
    buffer_executor: Arc<BufferNotificationExecutor>,
}

impl SendSocialNotifyExecutor {
    pub fn new(
        notification_service: Arc<dyn NotificationService>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_preferences_repo: Arc<dyn UserPreferencesRepository>,
        quiet_hours_resolver: Arc<QuietHoursResolver>,
        buffer_executor: Arc<BufferNotificationExecutor>,
    ) -> Self {
        Self {
            notification_service,
            user_socials_repo,
            user_preferences_repo,
            quiet_hours_resolver,
            buffer_executor,
        }
    }
}

impl CommandExecutor for SendSocialNotifyExecutor {
    type Command = SendSocialNotifyExecutorCommand;
    type Response = SendSocialNotifyExecutorResponse;
    type Error = SendSocialNotifyExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let owner_user_id = match self
            .user_socials_repo
            .find_by_social_chat_id(&cmd.chat_id, &cmd.social_type)
            .await
        {
            Ok(Some(account)) => Some(account.user_id),
            Ok(None) => None,
            Err(e) => {
                tracing::warn!(
                    chat_id = %cmd.chat_id.0,
                    social_type = ?cmd.social_type,
                    error = %e,
                    "Failed to lookup user by chat_id, sending without DND check"
                );
                None
            }
        };

        // Для личных чатов — берём prefs юзера. Для групповых — None (применяется
        // дефолтный DND из конфига через QuietHoursResolver).
        let prefs = match owner_user_id {
            Some(user_id) => self
                .user_preferences_repo
                .find_by_user_id(user_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        user_id = user_id.0,
                        error = %e,
                        "Failed to load user preferences, treating as default"
                    );
                    None
                }),
            None => None,
        };

        let now = Utc::now();
        if self.quiet_hours_resolver.is_quiet(prefs.as_ref(), now) {
            let deliver_after = self
                .quiet_hours_resolver
                .next_active_at(prefs.as_ref(), now);

            // Длинный отпуск → дропаем чтобы не накопить лавину к моменту возврата.
            let drop_threshold = now + Duration::hours(VACATION_DROP_THRESHOLD_HOURS);
            if deliver_after > drop_threshold {
                tracing::debug!(
                    chat_id = %cmd.chat_id.0,
                    user_id = ?owner_user_id.map(|u| u.0),
                    deliver_after = %deliver_after,
                    "Dropping notification (long vacation/away period)"
                );
                return Ok(SendSocialNotifyExecutorResponse {});
            }

            tracing::debug!(
                chat_id = %cmd.chat_id.0,
                user_id = ?owner_user_id.map(|u| u.0),
                deliver_after = %deliver_after,
                "Buffering notification due to quiet hours"
            );

            self.buffer_executor
                .execute(&BufferNotificationExecutorCommand {
                    user_id: owner_user_id,
                    social_type: cmd.social_type,
                    chat_id: cmd.chat_id,
                    message: cmd.message.clone(),
                    event_type: DEFAULT_EVENT_TYPE.to_string(),
                    deliver_after,
                })
                .await?;

            return Ok(SendSocialNotifyExecutorResponse {});
        }

        self.notification_service
            .send_message(&cmd.social_type, &cmd.chat_id, &cmd.message)
            .await
            .inspect_err(|e| {
                tracing::error!(
                    chat_id = %cmd.chat_id.0,
                    social_type = ?cmd.social_type,
                    error = %e,
                    "Failed to send notification"
                );
            })?;

        Ok(SendSocialNotifyExecutorResponse {})
    }
}
