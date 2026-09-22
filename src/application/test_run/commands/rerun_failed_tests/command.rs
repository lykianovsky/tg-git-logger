use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

/// Перезапуск только упавших тестов последнего прогона
pub struct RerunFailedTestsCommand {
    pub repository_id: RepositoryId,
    pub social_user_id: SocialUserId,
    pub chat_id: SocialChatId,
}
