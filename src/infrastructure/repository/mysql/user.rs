use crate::domain::user::entities::User;
use crate::domain::user::repository::{AssignUserRoleException, BindGitHubException, CreateUserException, FindUserByGitHubLoginException, FindUserByIdException, FindUserByTgIdException, UserRepository};
use crate::domain::user::value_objects::{UserGithubAccount, UserGithubId, UserId, UserRole, UserTelegramAccount, UserTelegramId};
use crate::infrastructure::database::mysql::entities::{git_hub_accounts, roles, telegram_accounts, user_roles, users};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait};
use std::sync::Arc;

pub struct MySQLUserRepository {
    pub db: Arc<DatabaseConnection>,
}

impl MySQLUserRepository {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Собираем доменную сущность User из моделей
    async fn build_domain_user(&self, mysql_user: users::Model) -> Result<User, sea_orm::DbErr> {
        // Telegram
        let telegram = telegram_accounts::Entity::find()
            .filter(telegram_accounts::Column::UserId.eq(mysql_user.id))
            .one(self.db.as_ref())
            .await?
            .map(|tg| UserTelegramAccount {
                telegram_id: UserTelegramId(tg.telegram_user_id as i32),
                username: tg.username,
                chat_id: 0,
            });

        // GitHub
        let github = git_hub_accounts::Entity::find()
            .filter(git_hub_accounts::Column::UserId.eq(mysql_user.id))
            .one(self.db.as_ref())
            .await?
            .map(|gh| UserGithubAccount {
                github_id: UserGithubId(gh.git_hub_id as u64),
                login: gh.login,
            });

        // Роли
        let role_models = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(mysql_user.id))
            .find_also_related(roles::Entity)
            .all(self.db.as_ref())
            .await?;

        let mut roles_vec = Vec::new();

        for (user_role, maybe_role) in role_models {
            if let Some(role) = maybe_role {
                roles_vec.push(UserRole::User);
            }
        }

        Ok(User {
            id: UserId(mysql_user.id),
            telegram,
            github,
            roles: roles_vec,
        })
    }
}

#[async_trait]
impl UserRepository for MySQLUserRepository {
    async fn create(&self, user: &User) -> Result<(), CreateUserException> {
        let txn = self.db.begin().await.map_err(|_| CreateUserException::DbError("Transaction start failed".into()))?;

        // Создаем пользователя
        let user_model = users::ActiveModel {
            is_active: Set(true),
            ..Default::default()
        };

        let inserted_user = user_model
            .insert(&txn)
            .await
            .map_err(|e| CreateUserException::DbError(e.to_string()))?;

        // Telegram
        if let Some(tg) = &user.telegram {
            telegram_accounts::ActiveModel {
                user_id: Set(inserted_user.id),
                telegram_user_id: Set(tg.telegram_id.0 as i64),
                username: Set(tg.username.clone()),
                ..Default::default()
            }
                .insert(&txn)
                .await
                .map_err(|e| CreateUserException::DbError(e.to_string()))?;
        }

        // GitHub
        if let Some(gh) = &user.github {
            git_hub_accounts::ActiveModel {
                user_id: Set(inserted_user.id),
                git_hub_id: Set(gh.github_id.0 as i64),
                login: Set(gh.login.clone()),
                ..Default::default()
            }
                .insert(&txn)
                .await
                .map_err(|e| CreateUserException::DbError(e.to_string()))?;
        }

        txn.commit().await.map_err(|e| CreateUserException::DbError(e.to_string()))?;
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<User, FindUserByIdException> {
        let user = users::Entity::find_by_id(id.0)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByIdException::DbError(e.to_string()))?
            .ok_or(FindUserByIdException::DbError("User not found".into()))?;

        self.build_domain_user(user)
            .await
            .map_err(|e| FindUserByIdException::DbError(e.to_string()))
    }

    async fn find_by_tg_id(&self, id: UserTelegramId) -> Result<User, FindUserByTgIdException> {
        let tg = telegram_accounts::Entity::find()
            .filter(telegram_accounts::Column::TelegramUserId.eq(id.0))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByTgIdException::DbError(e.to_string()))?
            .ok_or(FindUserByTgIdException::DbError("Telegram user not found".into()))?;

        let user = users::Entity::find_by_id(tg.user_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByTgIdException::DbError(e.to_string()))?
            .ok_or(FindUserByTgIdException::DbError("User not found".into()))?;

        self.build_domain_user(user)
            .await
            .map_err(|e| FindUserByTgIdException::DbError(e.to_string()))
    }

    async fn find_by_github_login(&self, id: String) -> Result<User, FindUserByGitHubLoginException> {
        let gh = git_hub_accounts::Entity::find()
            .filter(git_hub_accounts::Column::Login.eq(id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByGitHubLoginException::DbError(e.to_string()))?
            .ok_or(FindUserByGitHubLoginException::DbError("GitHub user not found".into()))?;

        let user = users::Entity::find_by_id(gh.user_id)
            .one(self.db.as_ref())
            .await
            .map_err(|e| FindUserByGitHubLoginException::DbError(e.to_string()))?
            .ok_or(FindUserByGitHubLoginException::DbError("User not found".into()))?;

        self.build_domain_user(user)
            .await
            .map_err(|e| FindUserByGitHubLoginException::DbError(e.to_string()))
    }

    async fn assign_role(&self, user_id: UserId, role: UserRole) -> Result<(), AssignUserRoleException> {
        let role = roles::Entity::find()
            .filter(roles::Column::Name.eq(role.to_str()))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                AssignUserRoleException::DbError(e.to_string())
            })?
            .ok_or(AssignUserRoleException::DbError("Role not found".into()))?;

        let exists = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(user_id.0))
            .filter(user_roles::Column::RoleId.eq(role.id))
            .one(self.db.as_ref())
            .await
            .map_err(|e| {
                AssignUserRoleException::DbError(e.to_string())
            })?;

        if exists.is_some() {
            return Ok(());
        }

        let user_role = user_roles::ActiveModel {
            user_id: Set(user_id.0),
            role_id: Set(role.id),
        };

        user_role
            .insert(self.db.as_ref())
            .await
            .map_err(|e| {
                AssignUserRoleException::DbError(e.to_string())
            })?;

        Ok(())
    }

    async fn bind_github(&self, user_id: UserId, github: UserGithubAccount) -> Result<(), BindGitHubException> {
        let model = git_hub_accounts::ActiveModel {
            user_id: Set(user_id.0),
            git_hub_id: Set(github.github_id.0 as i64),
            login: Set(github.login.clone()),
            access_token: Set(None),
            ..Default::default()
        };

        model
            .insert(self.db.as_ref())
            .await
            .map_err(|e| BindGitHubException::DbError(e.to_string()))?;

        Ok(())
    }
}
