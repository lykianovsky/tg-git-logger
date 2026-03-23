use crate::domain::user::entities::user::User;
use crate::domain::user::entities::user_social_account::UserSocialAccount;
use crate::domain::user::entities::user_vc_account::UserVersionControlAccount;

#[derive(Debug)]
pub struct RegisterUserViaOAuthExecutorResponse {
    pub user: User,
    pub user_social_account: UserSocialAccount,
    pub user_version_control_account: UserVersionControlAccount,
}
