use thiserror::Error;

#[derive(Error, Debug)]
pub enum CreateOAuthLinkExecutorError {
    #[error(
        "В данный момент авторизация недоступна, попробуйте позже, или обратитесь к администрации"
    )]
    UnknownError,

    #[error("Ваш аккаунт уже зарегистрирован, данная команда больше недоступна")]
    ExistRegisteredSocialAccountError,

    #[error("Мы не смогли создать пользователя, пожалуйста, попробуйте позже")]
    UserCreationError,

    #[error("Failed to parse URL: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("Failed to serialize state: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Cache error: {0}")]
    Cache(String),
}
