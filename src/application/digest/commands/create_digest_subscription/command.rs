use crate::domain::digest::value_objects::digest_type::DigestType;
use crate::domain::repository::value_objects::repository_id::RepositoryId;
use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct CreateDigestSubscriptionCommand {
    pub social_user_id: SocialUserId,
    pub repository_id: Option<RepositoryId>,
    pub digest_type: DigestType,
    pub send_hour: i8,
    pub send_minute: i8,
    pub day_of_week: Option<i8>,
}
