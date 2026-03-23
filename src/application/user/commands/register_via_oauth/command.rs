use crate::domain::auth::entities::oauth_state::OpenAuthorizationState;

#[derive(Debug, Clone)]
pub struct RegisterUserViaOAuthExecutorCommand {
    pub code: String,
    pub state: OpenAuthorizationState,
}
