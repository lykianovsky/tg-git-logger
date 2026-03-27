use crate::application::user::queries::get_user_roles_by_telegram_id::error::GetUserRolesByTelegramIdError;
use crate::application::user::queries::get_user_roles_by_telegram_id::query::GetUserRolesByTelegramIdQuery;
use crate::application::user::queries::get_user_roles_by_telegram_id::response::GetUserRolesByTelegramIdResponse;
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::{
    GetAllUserRolesError, UserHasRolesRepository,
};
use crate::domain::user::repositories::user_social_accounts_repository::{
    FindSocialServiceByIdError, UserSocialAccountsRepository,
};
use std::sync::Arc;

pub struct GetUserRolesByTelegramIdExecutor {
    pub user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    pub user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
}

impl GetUserRolesByTelegramIdExecutor {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    ) -> Self {
        Self {
            user_socials_repo,
            user_has_roles_repo,
        }
    }
}

impl CommandExecutor for GetUserRolesByTelegramIdExecutor {
    type Command = GetUserRolesByTelegramIdQuery;
    type Response = GetUserRolesByTelegramIdResponse;
    type Error = GetUserRolesByTelegramIdError;

    async fn execute(&self, cmd: &Self::Command) -> Result<Self::Response, Self::Error> {
        let social = self
            .user_socials_repo
            .find_by_social_user_id(&cmd.social_user_id)
            .await
            .map_err(|e| match e {
                FindSocialServiceByIdError::NotFound => GetUserRolesByTelegramIdError::UserNotFound,
                FindSocialServiceByIdError::DbError(msg) => {
                    GetUserRolesByTelegramIdError::DbError(msg)
                }
            })?;

        let roles = self
            .user_has_roles_repo
            .get_all(social.user_id)
            .await
            .map_err(|e| match e {
                GetAllUserRolesError::DbError(msg) => GetUserRolesByTelegramIdError::DbError(msg),
                GetAllUserRolesError::InvalidField(msg) => {
                    GetUserRolesByTelegramIdError::DbError(msg)
                }
            })?;

        Ok(GetUserRolesByTelegramIdResponse {
            roles: roles.into_iter().map(|r| r.name).collect(),
        })
    }
}
