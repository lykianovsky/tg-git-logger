use crate::application::user::queries::check_org_membership::error::CheckOrgMembershipError;
use crate::application::user::queries::check_org_membership::query::CheckOrgMembershipQuery;
use crate::application::user::queries::check_org_membership::response::CheckOrgMembershipResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_repository::{FindUserByIdError, UserRepository};
use crate::domain::user::repositories::user_social_accounts_repository::{
    FindSocialServiceByIdError, UserSocialAccountsRepository,
};
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::version_control::ports::version_control_client::VersionControlClient;
use crate::infrastructure::drivers::cache::contract::CacheService;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use std::sync::Arc;

const CACHE_TTL_SECS: u64 = 300;

pub struct CheckOrgMembershipExecutor {
    pub user_repo: Arc<dyn UserRepository>,
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    pub version_control_client: Arc<dyn VersionControlClient>,
    pub reversible_cipher: Arc<ReversibleCipher>,
    pub cache: Arc<dyn CacheService>,
    pub required_organization: Option<String>,
    pub admin_social_user_id: SocialUserId,
}

impl CheckOrgMembershipExecutor {
    fn cache_key(social_user_id: &SocialUserId, org: &str) -> String {
        format!("org_membership:{}:{}", org, social_user_id.0)
    }
}

impl CommandExecutor for CheckOrgMembershipExecutor {
    type Command = CheckOrgMembershipQuery;
    type Response = CheckOrgMembershipResponse;
    type Error = CheckOrgMembershipError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social = match self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await
        {
            Ok(s) => s,
            Err(FindSocialServiceByIdError::NotFound) => {
                return Ok(CheckOrgMembershipResponse::Allowed);
            }
            Err(FindSocialServiceByIdError::DbError(msg)) => {
                return Err(CheckOrgMembershipError::DbError(msg));
            }
        };

        match self.user_repo.find_by_id(social.user_id).await {
            Ok(user) => {
                if !user.is_active {
                    return Ok(CheckOrgMembershipResponse::Deactivated);
                }
            }
            Err(FindUserByIdError::NotFound) => {
                return Ok(CheckOrgMembershipResponse::Deactivated);
            }
            Err(FindUserByIdError::DbError(msg)) => {
                return Err(CheckOrgMembershipError::DbError(msg));
            }
        }

        let Some(org) = self.required_organization.as_deref().filter(|s| !s.is_empty()) else {
            return Ok(CheckOrgMembershipResponse::Allowed);
        };

        if cmd.social_user_id == self.admin_social_user_id {
            return Ok(CheckOrgMembershipResponse::Allowed);
        }

        let key = Self::cache_key(&cmd.social_user_id, org);

        match self.cache.get(&key).await {
            Ok(Some(cached)) => {
                if cached == "true" {
                    return Ok(CheckOrgMembershipResponse::Allowed);
                }
                if cached == "false" {
                    return Ok(CheckOrgMembershipResponse::Blocked {
                        organization: org.to_string(),
                    });
                }
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = %e, key = %key, "Cache read failed; proceeding to GitHub");
            }
        }

        let vc = self
            .user_vc_accounts_repo
            .find_by_user_id(&social.user_id)
            .await
            .map_err(|e| CheckOrgMembershipError::DbError(e.to_string()))?;

        let token = self.reversible_cipher.decrypt(vc.access_token.value())?;

        let is_member = self
            .version_control_client
            .is_user_in_organization(&token, org)
            .await
            .map_err(|e| CheckOrgMembershipError::CheckFailed(e.to_string()))?;

        let value = if is_member { "true" } else { "false" };
        if let Err(e) = self.cache.set(&key, value, CACHE_TTL_SECS).await {
            tracing::warn!(error = %e, key = %key, "Failed to cache membership result");
        }

        if is_member {
            Ok(CheckOrgMembershipResponse::Allowed)
        } else {
            Ok(CheckOrgMembershipResponse::Blocked {
                organization: org.to_string(),
            })
        }
    }
}
