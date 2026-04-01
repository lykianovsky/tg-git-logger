use crate::application::digest::commands::send_due_digests::command::SendDueDigestsCommand;
use crate::application::digest::commands::send_due_digests::error::SendDueDigestsExecutorError;
use crate::application::digest::commands::send_due_digests::response::SendDueDigestsResponse;
use crate::domain::digest::repositories::digest_subscription_repository::DigestSubscriptionRepository;
use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::utils::builder::message::MessageBuilder;
use chrono::{Datelike, Utc};
use std::sync::Arc;

pub struct SendDueDigestsExecutor {
    digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    notification_service: Arc<dyn NotificationService>,
}

impl SendDueDigestsExecutor {
    pub fn new(
        digest_subscription_repo: Arc<dyn DigestSubscriptionRepository>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            digest_subscription_repo,
            user_socials_repo,
            notification_service,
        }
    }
}

impl CommandExecutor for SendDueDigestsExecutor {
    type Command = SendDueDigestsCommand;
    type Response = SendDueDigestsResponse;
    type Error = SendDueDigestsExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .digest_subscription_repo
            .find_due(cmd.hour, cmd.minute)
            .await
            .map_err(|e| SendDueDigestsExecutorError::DbError(e.to_string()))?;

        let now = Utc::now();
        let today_weekday = now.weekday().num_days_from_monday() as i8;
        let mut sent_count = 0;

        for sub in &subscriptions {
            // Skip weekly digests if it's not the right day
            if sub.digest_type == DigestType::Weekly {
                if let Some(day) = sub.day_of_week {
                    if day != today_weekday {
                        continue;
                    }
                }
            }

            // Skip if already sent this minute
            if let Some(last_sent) = sub.last_sent_at {
                let diff = now.signed_duration_since(last_sent);
                if diff.num_minutes() < 1 {
                    continue;
                }
            }

            // Get user's social account for chat_id
            let social = match self
                .user_socials_repo
                .find_by_user_id(&sub.user_id)
                .await
            {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        error = %e,
                        user_id = sub.user_id.0,
                        "Failed to find social account for digest"
                    );
                    continue;
                }
            };

            let type_label = match sub.digest_type {
                DigestType::Daily => "ежедневный",
                DigestType::Weekly => "еженедельный",
            };

            let message = MessageBuilder::new()
                .bold(&format!("📬 Дайджест ({})", type_label))
                .empty_line()
                .line("Здесь будет сводка по активности в репозиториях.")
                .line("Функция генерации отчёта будет расширена.")
                .empty_line()
                .italic(&format!("⏰ {}", now.format("%d.%m.%Y %H:%M")));

            let chat_id = SocialChatId(social.social_user_id.0 as i64);

            if let Err(e) = self
                .notification_service
                .send_message(&SocialType::Telegram, &chat_id, &message)
                .await
            {
                tracing::error!(
                    error = %e,
                    user_id = sub.user_id.0,
                    "Failed to send digest notification"
                );
                continue;
            }

            // Update last_sent_at
            let mut updated_sub = sub.clone();
            updated_sub.last_sent_at = Some(now);

            if let Err(e) = self.digest_subscription_repo.update(&updated_sub).await {
                tracing::error!(
                    error = %e,
                    subscription_id = sub.id.0,
                    "Failed to update last_sent_at for digest"
                );
            }

            sent_count += 1;
        }

        Ok(SendDueDigestsResponse { sent_count })
    }
}
