use crate::domain::user::repositories::user_social_accounts_repository::UserSocialAccountsRepository;
use crate::domain::user::repositories::user_vc_accounts_repository::UserVersionControlAccountsRepository;
use crate::domain::user::value_objects::social_user_id::SocialUserId;
use crate::domain::user::value_objects::user_id::UserId;
use crate::utils::security::crypto::reversible::ReversibleCipher;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveCiTokenError {
    #[error("User has no linked version control account")]
    NoVersionControlAccount,

    #[error("Failed to decrypt access token: {0}")]
    DecryptError(String),
}

/// Чей токен идёт в CI: прогон из чата запускается правами того, кто нажал кнопку,
/// а расписание и добор итогов — правами администратора бота. Отдельный сервисный
/// токен не нужен: у привязанных аккаунтов уже есть доступ к репозиториям.
pub struct CiTokenResolver {
    user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
    user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
    reversible_cipher: Arc<ReversibleCipher>,
    admin_social_user_id: SocialUserId,
}

/// Кто запускает прогон и чьим токеном мы ходим в CI
pub struct CiActor {
    pub user_id: UserId,
    pub token: String,
}

impl CiTokenResolver {
    pub fn new(
        user_socials_repo: Arc<dyn UserSocialAccountsRepository>,
        user_vc_accounts_repo: Arc<dyn UserVersionControlAccountsRepository>,
        reversible_cipher: Arc<ReversibleCipher>,
        admin_social_user_id: SocialUserId,
    ) -> Self {
        Self {
            user_socials_repo,
            user_vc_accounts_repo,
            reversible_cipher,
            admin_social_user_id,
        }
    }

    /// Токен пользователя чата: им запускаются прогоны из бота
    pub async fn resolve_by_social_user_id(
        &self,
        social_user_id: &SocialUserId,
    ) -> Result<CiActor, ResolveCiTokenError> {
        let social = self
            .user_socials_repo
            .find_by_social_user_id(social_user_id)
            .await
            .map_err(|_| ResolveCiTokenError::NoVersionControlAccount)?;

        self.resolve_by_user_id(&social.user_id).await
    }

    pub async fn resolve_by_user_id(
        &self,
        user_id: &UserId,
    ) -> Result<CiActor, ResolveCiTokenError> {
        let account = self
            .user_vc_accounts_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| ResolveCiTokenError::NoVersionControlAccount)?;

        let token = self
            .reversible_cipher
            .decrypt(account.access_token.value())
            .map_err(|error| ResolveCiTokenError::DecryptError(error.to_string()))?;

        Ok(CiActor {
            user_id: account.user_id,
            token,
        })
    }

    /// Прогон без инициатора — ночной или добор итогов: идём правами администратора
    pub async fn resolve_for_background(&self) -> Result<CiActor, ResolveCiTokenError> {
        self.resolve_by_social_user_id(&self.admin_social_user_id)
            .await
    }

    pub async fn resolve_or_background(
        &self,
        user_id: Option<UserId>,
    ) -> Result<CiActor, ResolveCiTokenError> {
        match user_id {
            Some(user_id) => self.resolve_by_user_id(&user_id).await,
            None => self.resolve_for_background().await,
        }
    }
}
