use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegisterUserViaOAuthExecutorError {
    #[error(
        "В данный момент авторизация недоступна, попробуйте позже, или обратитесь к администрации"
    )]
    UnknownError,

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Invalid state error")]
    InvalidState,
}
