use crate::config::environment::ENV;
use crate::domain::user::entities::User;
use crate::domain::user::repository::{AssignUserRoleException, CreateUserException, FindUserByTgIdException, UserRepository};
use crate::domain::user::value_objects::{UserId, UserRole, UserTelegramAccount, UserTelegramId};
use std::sync::Arc;

#[derive(Debug)]
pub enum CreateAdminUserError {
    CreateUser(CreateUserException),
    AssignRole(AssignUserRoleException),
    FindUserByTgId(FindUserByTgIdException),
    HasExist(String)
}

pub struct CreateUserByTelegramUserUseCase {
    user_repo: Arc<dyn UserRepository + Send + Sync>,
}

impl CreateUserByTelegramUserUseCase {
    pub fn new(user_repo: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { user_repo }
    }

    /// Создать пользователя по Telegram ID и выдать роль admin
    pub async fn execute(
        &self,
        telegram_id: UserTelegramId,
        chat_id: i64,
        username: Option<String>,
    ) -> Result<User, CreateAdminUserError> {
        // 1️⃣ Проверяем, есть ли уже пользователь
        if let Ok(existing_user) = self.user_repo.find_by_tg_id(telegram_id).await {
            // Если нашли — возвращаем ошибку
            return Err(CreateAdminUserError::HasExist(format!(
                "User with Telegram ID {} already exists",
                telegram_id.0
            )));
        }

        // 2️⃣ Создаем нового пользователя
        let new_user = User {
            id: UserId(0),
            telegram: Some(UserTelegramAccount {
                telegram_id,
                username,
                chat_id,
            }),
            github: None,
            roles: vec![],
        };

        self.user_repo
            .create(&new_user)
            .await
            .map_err(CreateAdminUserError::CreateUser)?;

        // 3️⃣ Получаем вновь созданного пользователя
        let user = self
            .user_repo
            .find_by_tg_id(telegram_id)
            .await
            .map_err(CreateAdminUserError::FindUserByTgId)?;

        // 4️⃣ Если Telegram ID совпадает с админским — выдаем роль Admin
        let admin_tg_id: i32 = ENV.get("TELEGRAM_ADMIN_USER_ID").parse().unwrap();
        if telegram_id.0 == admin_tg_id {
            self.user_repo
                .assign_role(user.id.clone(), UserRole::Admin)
                .await
                .map_err(CreateAdminUserError::AssignRole)?;
        }

        Ok(user)
    }

}
