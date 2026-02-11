#[derive(Debug, Clone)]
pub struct RegisterUserViaOAuthExecutorCommand {
    pub code: String,
    pub state: String,
}
