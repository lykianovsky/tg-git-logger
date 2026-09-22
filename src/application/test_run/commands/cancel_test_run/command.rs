use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

/// Отмена идущего прогона: отменяем правами того, кто нажал кнопку
pub struct CancelTestRunCommand {
    pub repository_id: RepositoryId,
    pub social_user_id: SocialUserId,
}
