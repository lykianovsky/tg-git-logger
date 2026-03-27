use crate::domain::repository::value_objects::repository_id::RepositoryId;

pub struct UnsetRepositoryNotificationChatCommand {
    pub repository_id: RepositoryId,
}
