use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct ListTestBlocksQuery {
    pub repository_id: RepositoryId,
    /// Каталог тестов читаем правами того, кто открыл список
    pub social_user_id: SocialUserId,
}
