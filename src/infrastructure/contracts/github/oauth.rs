use crate::domain::user::value_objects::UserTelegramId;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubOAuthState {
    pub telegram_id: UserTelegramId,
    pub chat_id: i64,
}
