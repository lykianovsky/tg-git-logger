use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;

pub struct SetRepositoryNotificationsChatCommand {
    pub repository_id: RepositoryId,
    pub notifications_chat_id: SocialChatId,
}
