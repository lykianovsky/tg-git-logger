use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct GetUserOverviewQuery {
    pub social_user_id: SocialUserId,
}
