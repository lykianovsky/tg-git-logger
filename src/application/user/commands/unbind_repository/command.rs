use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct UnbindRepositoryCommand {
    pub social_user_id: SocialUserId,
    pub repository_id: RepositoryId,
}
