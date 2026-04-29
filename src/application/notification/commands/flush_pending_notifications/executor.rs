use crate::application::notification::commands::flush_pending_notifications::command::FlushPendingNotificationsExecutorCommand;
use crate::application::notification::commands::flush_pending_notifications::error::FlushPendingNotificationsExecutorError;
use crate::application::notification::commands::flush_pending_notifications::response::FlushPendingNotificationsExecutorResponse;
use crate::domain::notification::services::notification_service::NotificationService;
use crate::domain::pending_notification::entities::pending_notification::PendingNotification;
use crate::domain::pending_notification::repositories::pending_notification_repository::PendingNotificationsRepository;
use crate::domain::pending_notification::value_objects::pending_notification_id::PendingNotificationId;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::user::value_objects::user_id::UserId;
use crate::utils::builder::message::MessageBuilder;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

pub struct FlushPendingNotificationsExecutor {
    repo: Arc<dyn PendingNotificationsRepository>,
    notification_service: Arc<dyn NotificationService>,
}

impl FlushPendingNotificationsExecutor {
    pub fn new(
        repo: Arc<dyn PendingNotificationsRepository>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            repo,
            notification_service,
        }
    }
}

type GroupKey = (Option<UserId>, SocialType, SocialChatId);

impl CommandExecutor for FlushPendingNotificationsExecutor {
    type Command = FlushPendingNotificationsExecutorCommand;
    type Response = FlushPendingNotificationsExecutorResponse;
    type Error = FlushPendingNotificationsExecutorError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let now = Utc::now();
        let due = self.repo.find_due(now).await?;

        if due.is_empty() {
            return Ok(FlushPendingNotificationsExecutorResponse { flushed_count: 0 });
        }

        let mut groups: HashMap<GroupKey, Vec<PendingNotification>> = HashMap::new();
        for notif in due {
            let key = (notif.user_id, notif.social_type, notif.social_chat_id);
            groups.entry(key).or_default().push(notif);
        }

        let mut delivered_ids: Vec<PendingNotificationId> = Vec::new();

        for ((user_id, social_type, chat_id), notifications) in groups {
            let combined = build_combined_message(&notifications);
            let ids: Vec<PendingNotificationId> = notifications.iter().map(|n| n.id).collect();

            match self
                .notification_service
                .send_message(&social_type, &chat_id, &combined)
                .await
            {
                Ok(_) => {
                    delivered_ids.extend(ids);
                }
                Err(e) => {
                    tracing::error!(
                        user_id = ?user_id.map(|u| u.0),
                        chat_id = %chat_id.0,
                        error = %e,
                        "Failed to flush pending notifications group, will retry next tick"
                    );
                }
            }
        }

        let count = delivered_ids.len();
        if !delivered_ids.is_empty() {
            self.repo.delete_many(&delivered_ids).await?;
        }

        Ok(FlushPendingNotificationsExecutorResponse {
            flushed_count: count,
        })
    }
}

// TG лимит сообщения 4096 — оставляем запас на overflow-маркер.
const MAX_COMBINED_LEN: usize = 3800;

fn build_combined_message(notifications: &[PendingNotification]) -> MessageBuilder {
    let mut builder = MessageBuilder::new()
        .bold(&t!("telegram_bot.notifications.pending_digest.title").to_string())
        .empty_line();

    for (idx, notif) in notifications.iter().enumerate() {
        if idx > 0 {
            builder = builder.line("———");
        }
        builder = builder.raw(&notif.message.clone().build());
    }

    builder.with_max_length(MAX_COMBINED_LEN)
}
