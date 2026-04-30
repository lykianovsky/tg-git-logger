use crate::domain::user::value_objects::social_user_id::SocialUserId;

pub struct GetPendingReviewsQuery {
    pub social_user_id: SocialUserId,
}
