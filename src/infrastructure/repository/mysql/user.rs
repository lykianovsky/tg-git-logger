use crate::domain::user::entities::User;
use crate::domain::user::repository::{CreateUserException, FindUserByGitHubIdException, FindUserByIdException, FindUserByTgIdException, UserRepository};
use crate::domain::user::value_objects::{UserGithubId, UserId, UserTelegramId};
use crate::infrastructure::database::mysql::entities::{github_account, roles, telegram_account, user as user_entity, user_role};
use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionTrait};

pub struct MySQLUserRepository {
    pub db: DatabaseConnection,
}

#[async_trait]
impl UserRepository for MySQLUserRepository {
    async fn create(&self, user: &User) -> Result<(), CreateUserException> {
        let txn = self.db.begin().await.map_err(|e| CreateUserException::DbError(e.to_string()))?;

        // 1️⃣ Вставляем User
        let user_model = user_entity::ActiveModel {
            ..Default::default() // id автоинкремент
        };
        let user_model = user_model.insert(&txn).await.map_err(|e| CreateUserException::DbError(e.to_string()))?;

        let user_id = user_model.id;

        // 2️⃣ Вставляем TelegramAccount, если есть
        if let Some(telegram) = &user.telegram {
            let tg_model = telegram_account::ActiveModel {
                user_id: Set(user_id),
                telegram_id: Set(telegram.telegram_id.0),
                username: Set(telegram.username.clone()),
                chat_id: Set(telegram.chat_id),
                ..Default::default()
            };
            tg_model.insert(&txn).await.map_err(|e| CreateUserException::DbError(e.to_string()))?;
        }

        // 3️⃣ Вставляем GithubAccount, если есть
        if let Some(github) = &user.github {
            let gh_model = github_account::ActiveModel {
                user_id: Set(user_id),
                github_id: Set(github.github_id.0),
                login: Set(github.login.clone()),
                ..Default::default()
            };
            gh_model.insert(&txn).await.map_err(|e| CreateUserException::DbError(e.to_string()))?;
        }

        // 4️⃣ Вставляем роли
        for role in &user.roles {
            // Сначала получаем id роли по имени
            let role_name = role.to_str();

            let role_model = roles::Entity::find()
                .filter(roles::Column::Name.eq(role_name))
                .one(&txn)
                .await
                .map_err(|e| CreateUserException::DbError(e.to_string()))?;

            let role_model = match role_model {
                Some(r) => r,
                None => return Err(CreateUserException::DbError(format!("Role {} not found", role_name))),
            };

            let ur_model = user_role::ActiveModel {
                user_id: Set(user_id),
                role_id: Set(role_model.id),
                ..Default::default()
            };
            ur_model.insert(&txn).await.map_err(|e| CreateUserException::DbError(e.to_string()))?;
        }

        txn.commit().await.map_err(|e| CreateUserException::DbError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_github_id(&self, id: UserGithubId) -> Result<User, FindUserByGitHubIdException> {
        todo!()
    }

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdException> {
        todo!()
    }

    async fn find_by_tg_id(&self, id: UserTelegramId) -> Result<User, FindUserByTgIdException> {
        todo!()
    }
}
