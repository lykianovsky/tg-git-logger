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
        tracing::debug!(hour = cmd.hour, minute = cmd.minute, "Looking for due digests");

        let subscriptions = self
            .digest_subscription_repo
            .find_due(cmd.hour, cmd.minute)
            .await
            .map_err(|e| SendDueDigestsExecutorError::DbError(e.to_string()))?;

        tracing::debug!(count = subscriptions.len(), "Found due subscriptions");

        let now = Utc::now();
        let today_weekday = now.weekday().num_days_from_monday() as i8;
        let mut sent_count = 0;

        for sub in &subscriptions {
            tracing::debug!(
                subscription_id = sub.id.0,
                user_id = sub.user_id.0,
                digest_type = sub.digest_type.as_str(),
                day_of_week = ?sub.day_of_week,
                last_sent_at = ?sub.last_sent_at,
                "Processing subscription"
            );

            // Skip weekly digests if it's not the right day
            if sub.digest_type == DigestType::Weekly {
                if let Some(day) = sub.day_of_week {
                    if day != today_weekday {
                        tracing::debug!(
                            subscription_id = sub.id.0,
                            expected_day = day,
                            today = today_weekday,
                            "Skipping weekly digest — wrong day"
                        );
                        continue;
                    }
                }
            }

            // Skip if already sent this minute
            if let Some(last_sent) = sub.last_sent_at {
                let diff = now.signed_duration_since(last_sent);
                if diff.num_minutes() < 1 {
                    tracing::debug!(
                        subscription_id = sub.id.0,
                        "Skipping — already sent this minute"
                    );
                    continue;
                }
            }

            // Get user's social account for chat_id
            let social = match self
                .user_socials_repo
                .find_by_user_id(&sub.user_id)
                .await
            {
                Ok(s) => {
                    tracing::debug!(
                        user_id = sub.user_id.0,
                        social_user_id = s.social_user_id.0,
                        "Found social account"
                    );
                    s
                }
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

            tracing::debug!(
                chat_id = chat_id.0,
                subscription_id = sub.id.0,
                "Sending digest notification"
            );

            if let Err(e) = self
                .notification_service
                .send_message(&SocialType::Telegram, &chat_id, &message)
                .await
            {
                tracing::error!(
                    error = %e,
                    user_id = sub.user_id.0,
                    chat_id = chat_id.0,
                    "Failed to send digest notification"
                );
                continue;
            }

            tracing::debug!(subscription_id = sub.id.0, "Digest sent, updating last_sent_at");

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
