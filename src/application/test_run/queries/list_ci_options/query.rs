use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

/// Что показываем на шаге подключения тестов
pub enum CiOptionKind {
    Workflows,
}

pub struct ListCiOptionsQuery {
    pub repository_id: RepositoryId,
    pub social_user_id: SocialUserId,
    pub kind: CiOptionKind,
}
