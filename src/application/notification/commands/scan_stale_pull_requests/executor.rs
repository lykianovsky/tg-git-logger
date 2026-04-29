use crate::application::notification::commands::scan_stale_pull_requests::command::ScanStalePullRequestsExecutorCommand;
use crate::application::notification::commands::scan_stale_pull_requests::error::ScanStalePullRequestsExecutorError;
use crate::application::notification::commands::scan_stale_pull_requests::response::ScanStalePullRequestsExecutorResponse;
use crate::delivery::jobs::consumers::send_social_notify::payload::SendSocialNotifyJob;
use crate::domain::pr_review::repositories::pr_review_repository::PrReviewRepository;
use crate::domain::repository::repositories::repository_repository::RepositoryRepository;
use crate::domain::role::value_objects::role_name::RoleName;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::domain::version_control::ports::version_control_client::{
    OpenPullRequestSummary, VersionControlClient,
};
use crate::infrastructure::drivers::message_broker::contracts::publisher::MessageBrokerPublisher;
use crate::utils::builder::message::MessageBuilder;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use chrono::{Duration, Utc};
use std::sync::Arc;

pub struct ScanStalePullRequestsExecutor {
    pub publisher: Arc<dyn MessageBrokerPublisher>,
    pub repository_repo: Arc<dyn RepositoryRepository>,
    pub pr_review_repo: Arc<dyn PrReviewRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub reversible_cipher: Arc<ReversibleCipher>,
    pub stale_threshold_hours: i64,
}

impl ScanStalePullRequestsExecutor {
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

    async fn resolve_tg_username(&self, github_login: &str) -> Option<String> {
        let vc = self
            .user_vc_accounts_repo
            .find_by_login(github_login)
            .await
            .ok()?;
        let social = self
            .user_socials_repo
            .find_by_user_id(&vc.user_id)
            .await
            .ok()?;
        social.social_user_login
    }
}

impl CommandExecutor for ScanStalePullRequestsExecutor {
    type Command = ScanStalePullRequestsExecutorCommand;
    type Response = ScanStalePullRequestsExecutorResponse;
    type Error = ScanStalePullRequestsExecutorError;

    async fn execute(&self, _cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let token = match self.admin_token().await {
            Some(t) => t,
            None => {
                tracing::debug!("No admin token available — skipping stale PR scan");
                return Ok(ScanStalePullRequestsExecutorResponse {
                    repos_scanned: 0,
                    stale_count: 0,
                });
            }
        };

        let repos = self
            .repository_repo
            .find_all()
            .await
            .map_err(|e| ScanStalePullRequestsExecutorError::DbError(e.to_string()))?;

        let now = Utc::now();
        let threshold = Duration::hours(self.stale_threshold_hours);
        let mut stale_total = 0usize;
        let mut repos_scanned = 0usize;

        for repo in repos {
            let chat_id = match repo.notifications_chat_id.or(repo.social_chat_id) {
                Some(c) => c,
                None => continue,
            };

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
                        "Failed to list open PRs"
                    );
                    continue;
                }
            };
            repos_scanned += 1;

            let aged: Vec<&OpenPullRequestSummary> = prs
                .iter()
                .filter(|pr| now.signed_duration_since(pr.updated_at) > threshold)
                .collect();
            if aged.is_empty() {
                continue;
            }

            let repo_full = format!("{}/{}", repo.owner, repo.name);
            let mut stale: Vec<&OpenPullRequestSummary> = Vec::new();
            for pr in aged {
                match self.pr_review_repo.find_by_pr(&repo_full, pr.number).await {
                    Ok(reviews) => {
                        let has_approve = reviews
                            .iter()
                            .any(|r| r.last_review_state.eq_ignore_ascii_case("approved"));
                        if has_approve {
                            continue;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            repo = %repo_full,
                            pr = pr.number,
                            error = %e,
                            "Failed to fetch PR reviews; including PR in digest"
                        );
                    }
                }
                stale.push(pr);
            }
            if stale.is_empty() {
                continue;
            }
            stale_total += stale.len();

            let mut msg = MessageBuilder::new()
                .bold(&t!("telegram_bot.notifications.stale_pr_digest.title").to_string())
                .empty_line();

            for pr in &stale {
                let age = now.signed_duration_since(pr.updated_at);
                let age_label = format_age(age);

                let mut tags: Vec<String> = Vec::new();
                for login in &pr.requested_reviewers {
                    let display = match self.resolve_tg_username(login).await {
                        Some(tg) => format!("@{}", tg),
                        None => login.clone(),
                    };
                    tags.push(display);
                }
                let tag_line = if tags.is_empty() {
                    "—".to_string()
                } else {
                    tags.join(", ")
                };

                msg = msg.raw(&format!(
                    "• #{} <a href=\"{}\">{}</a> — {} ({})\n",
                    pr.number,
                    MessageBuilder::escape_html(&pr.url),
                    MessageBuilder::escape_html(&pr.title),
                    age_label,
                    MessageBuilder::escape_html(&tag_line),
                ));
            }

            self.publisher
                .publish(&SendSocialNotifyJob {
                    social_type: SocialType::Telegram,
                    chat_id,
                    message: msg,
                })
                .await
                .ok();
        }

        Ok(ScanStalePullRequestsExecutorResponse {
            repos_scanned,
            stale_count: stale_total,
        })
    }
}

fn format_age(d: Duration) -> String {
    let hours = d.num_hours();
    if hours < 24 {
        format!("{} ч", hours)
    } else {
        format!("{} дн", hours / 24)
    }
}
