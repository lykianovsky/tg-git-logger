use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;

pub struct SetRepositoryNotificationChatCommand {
    pub repository_id: RepositoryId,
    pub social_chat_id: SocialChatId,
}
