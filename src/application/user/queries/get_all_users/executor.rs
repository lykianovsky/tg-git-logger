use crate::application::user::queries::get_all_users::error::GetAllUsersExecutorError;
use crate::application::user::queries::get_all_users::query::GetAllUsersQuery;
use crate::application::user::queries::get_all_users::response::{
    GetAllUsersResponse, UserListItem,
};
use crate::domain::shared::command::CommandExecutor;
use crate::domain::user::repositories::user_has_roles_repository::UserHasRolesRepository;
use crate::domain::user::repositories::user_repository::UserRepository;
use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use std::sync::Arc;

pub struct GetAllUsersExecutor {
    user_repo: Arc<dyn UserRepository>,
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
}

impl GetAllUsersExecutor {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_has_roles_repo: Arc<dyn UserHasRolesRepository>,
    ) -> Self {
        Self {
            user_repo,
            user_socials_repo,
            user_has_roles_repo,
        }
    }
}

impl CommandExecutor for GetAllUsersExecutor {
    type Command = GetAllUsersQuery;
    type Response = GetAllUsersResponse;
    type Error = GetAllUsersExecutorError;

    async fn execute(
        &self,
        _cmd: &GetAllUsersQuery,
    ) -> Result<GetAllUsersResponse, GetAllUsersExecutorError> {
        let users = self
            .user_repo
            .find_all()
            .await
            .map_err(|e| GetAllUsersExecutorError::DbError(e.to_string()))?;

        let mut items = Vec::with_capacity(users.len());

        for user in &users {
            let social = self.user_socials_repo.find_by_user_id(&user.id).await.ok();

            let roles = self
                .user_has_roles_repo
                .get_all(user.id)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|r| r.name)
                .collect();

            items.push(UserListItem {
                user_id: user.id,
                is_active: user.is_active,
                social_login: social.as_ref().and_then(|s| s.social_user_login.clone()),
                social_user_id: social.as_ref().map(|s| s.social_user_id.0),
                roles,
                created_at: user.create_at,
            });
        }

        Ok(GetAllUsersResponse { users: items })
    }
}
