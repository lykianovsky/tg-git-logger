use crate::application::notification::commands::scan_pr_conflicts::command::ScanPrConflictsExecutorCommand;
use crate::application::notification::commands::scan_pr_conflicts::error::ScanPrConflictsExecutorError;
use crate::application::notification::commands::scan_pr_conflicts::response::ScanPrConflictsExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::notification_log::repositories::notification_log_repository::NotificationLogRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use chrono::{Duration, Utc};
use std::sync::Arc;

const KIND: &str = "pr_conflict";
const COOLDOWN_HOURS: i64 = 24;

pub struct ScanPrConflictsExecutor {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub notification_log_repo: Arc<dyn NotificationLogRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub reversible_cipher: Arc<ReversibleCipher>,
}

impl ScanPrConflictsExecutor {
    async fn admin_token(&self) -> Option<String> {
        let admin_ids = self
            .user_has_roles_repo
            .find_user_ids_by_role(RoleName::Admin)
            .await
            .ok()?;
        for user_id in admin_ids {
            if let Ok(vc) = self.user_vc_accounts_repo.find_by_user_id(&user_id).await {
                if let Ok(token) = self.reversible_cipher.decrypt(vc.access_token.value()) {
                    return Some(token);
                }
            }
        }
        None
    }
}

impl CommandExecutor for ScanPrConflictsExecutor {
    type Command = ScanPrConflictsExecutorCommand;
    type Response = ScanPrConflictsExecutorResponse;
    type Error = ScanPrConflictsExecutorError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let token = match self.admin_token().await {
            Some(t) => t,
            None => {
                tracing::debug!("No admin token available — skipping PR conflict scan");
                return Ok(ScanPrConflictsExecutorResponse {
                    repos_scanned: 0,
                    conflicts_count: 0,
                });
            }
        };

        let repos = self
            .repository_repo
            .find_all()
            .await
            .map_err(|e| ScanPrConflictsExecutorError::DbError(e.to_string()))?;

        let mut conflicts_total = 0usize;
        let mut repos_scanned = 0usize;
        let since = Utc::now() - Duration::hours(COOLDOWN_HOURS);

        for repo in repos {
            let prs = match self
                .version_control_client
                .list_open_pull_requests(&token, &repo.owner, &repo.name)
                .await
            {
                Ok(prs) => prs,
                Err(e) => {
                    tracing::warn!(
                        repo = %format!("{}/{}", repo.owner, repo.name),
                        error = %e,
                        "Failed to list open PRs for conflict scan"
                    );
                    continue;
                }
            };
            repos_scanned += 1;

            let repo_full = format!("{}/{}", repo.owner, repo.name);

            for pr in prs {
                let mergeable = match self
                    .version_control_client
                    .get_pr_mergeable_state(&token, &repo.owner, &repo.name, pr.number)
                    .await
                {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::debug!(
                            repo = %repo_full,
                            pr = pr.number,
                            error = %e,
                            "Failed to fetch PR mergeable_state"
                        );
                        continue;
                    }
                };

                if mergeable.as_deref() != Some("dirty") {
                    continue;
                }

                let vc_account = match self
                    .user_vc_accounts_repo
                    .find_by_login(&pr.author_login)
                    .await
                {
                    Ok(a) => a,
                    Err(_) => continue,
                };
                let social = match self
                    .user_socials_repo
                    .find_by_user_id(&vc_account.user_id)
                    .await
                {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                let key = format!("{}:{}", repo_full, pr.number);

                match self
                    .notification_log_repo
                    .was_sent_within(vc_account.user_id, KIND, &key, since)
                    .await
                {
                    Ok(true) => continue,
                    Ok(false) => {}
                    Err(e) => {
                        tracing::warn!(error = %e, "Failed dedup check for conflict cron");
                        continue;
                    }
                }

                let mut msg = MessageBuilder::new()
                    .bold(&t!("telegram_bot.notifications.pr_conflict.title").to_string())
                    .empty_line()
                    .with_html_escape(true)
                    .section(
                        &t!("telegram_bot.notifications.pr_conflict.pr").to_string(),
                        &format!("#{} — {}", pr.number, pr.title),
                    )
                    .section(
                        &t!("telegram_bot.notifications.pr_conflict.repository").to_string(),
                        &repo_full,
                    )
                    .with_html_escape(false);

                if !pr.url.is_empty() {
                    msg = msg.empty_line().raw(&format!(
                        "<a href=\"{}\">{}</a>",
                        MessageBuilder::escape_html(&pr.url),
                        t!("telegram_bot.notifications.pr_conflict.open").to_string()
                    ));
                }

                self.publisher
                    .publish(&SendSocialNotifyJob {
                        social_type: SocialType::Telegram,
                        chat_id: social.social_chat_id,
                        message: msg,
                    })
                    .await
                    .ok();

                if let Err(e) = self
                    .notification_log_repo
                    .record_sent(vc_account.user_id, KIND, &key)
                    .await
                {
                    tracing::warn!(error = %e, "Failed to record conflict cron notification");
                }

                conflicts_total += 1;
            }
        }

        Ok(ScanPrConflictsExecutorResponse {
            repos_scanned,
            conflicts_count: conflicts_total,
        })
    }
}
