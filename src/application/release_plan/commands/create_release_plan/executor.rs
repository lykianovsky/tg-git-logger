use crate::application::release_plan::commands::create_release_plan::command::CreateReleasePlanExecutorCommand;
use crate::application::release_plan::commands::create_release_plan::error::CreateReleasePlanExecutorError;
use crate::application::release_plan::commands::create_release_plan::response::CreateReleasePlanExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::release_plan::entities::release_plan::NewReleasePlan;
use crate::domain::release_plan::repositories::release_plan_repository::ReleasePlanRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use std::sync::Arc;

pub struct CreateReleasePlanExecutor {
    pub release_plan_repo: Arc<dyn ReleasePlanRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub publisher: Arc<dyn MessageBrokerPublisher>,
}

impl CommandExecutor for CreateReleasePlanExecutor {
    type Command = CreateReleasePlanExecutorCommand;
    type Response = CreateReleasePlanExecutorResponse;
    type Error = CreateReleasePlanExecutorError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let creator_social = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.created_by_social_user_id)
            .await?;

        // Если announce_chat_id явно не задан — резолвим из первого репо
        // (notifications_chat_id → social_chat_id → None).
        let mut announce_chat_id = cmd.announce_chat_id;
        if announce_chat_id.is_none() {
            for rid in &cmd.repository_ids {
                if let Ok(repo) = self.repository_repo.find_by_id(*rid).await {
                    if let Some(chat) = repo.notifications_chat_id.or(repo.social_chat_id) {
                        announce_chat_id = Some(chat);
                        break;
                    }
                }
            }
        }

        let new_plan = NewReleasePlan {
            planned_date: cmd.planned_date,
            call_datetime: cmd.call_datetime,
            meeting_url: cmd.meeting_url.clone(),
            note: cmd.note.clone(),
            announce_chat_id,
            repository_ids: cmd.repository_ids.clone(),
            created_by_user_id: creator_social.user_id,
        };

        let plan = self.release_plan_repo.create(&new_plan).await?;

        if let Some(chat_id) = announce_chat_id {
            let mut repo_names: Vec<String> = Vec::new();
            for rid in &cmd.repository_ids {
                if let Ok(repo) = self.repository_repo.find_by_id(*rid).await {
                    repo_names.push(format!("{}/{}", repo.owner, repo.name));
                }
            }
            let creator_label = creator_social
                .social_user_login
                .map(|n| format!("@{}", n))
                .unwrap_or_else(|| creator_social.social_user_id.0.to_string());

            let mut msg = MessageBuilder::new()
                .bold(&t!("telegram_bot.notifications.release_plan.created").to_string())
                .empty_line()
                .with_html_escape(true)
                .section(
                    &t!("telegram_bot.notifications.release_plan.repos").to_string(),
                    &if repo_names.is_empty() {
                        "—".to_string()
                    } else {
                        repo_names.join(", ")
                    },
                )
                .section(
                    &t!("telegram_bot.notifications.release_plan.date").to_string(),
                    &cmd.planned_date.format("%d.%m.%Y").to_string(),
                );

            if let Some(call) = cmd.call_datetime {
                msg = msg.section(
                    &t!("telegram_bot.notifications.release_plan.call").to_string(),
                    &call.format("%d.%m.%Y %H:%M UTC").to_string(),
                );
            }
            if let Some(url) = &cmd.meeting_url {
                msg = msg.with_html_escape(false).raw(&format!(
                    "🔗 <a href=\"{}\">{}</a>\n",
                    MessageBuilder::escape_html(url),
                    t!("telegram_bot.notifications.release_plan.meeting").to_string()
                ));
                msg = msg.with_html_escape(true);
            }
            if let Some(note) = &cmd.note {
                msg = msg.section(
                    &t!("telegram_bot.notifications.release_plan.note").to_string(),
                    note,
                );
            }
            msg = msg.section(
                &t!("telegram_bot.notifications.release_plan.created_by").to_string(),
                &creator_label,
            );

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id,
                    message: msg,
                })
                .await
                .ok();
        }

        Ok(CreateReleasePlanExecutorResponse { plan })
    }
}
