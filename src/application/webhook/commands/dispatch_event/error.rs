use thiserror::Error;

#[derive(Error, Debug)]
pub enum DispatchWebhookEventExecutorError {
    #[error(
        "В данный момент авторизация недоступна, попробуйте позже, или обратитесь к администрации"
    )]
    UnknownError,
}
