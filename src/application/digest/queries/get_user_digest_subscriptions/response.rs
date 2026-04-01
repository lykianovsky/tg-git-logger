use crate::domain::digest::entities::digest_subscription::DigestSubscription;

pub struct GetUserDigestSubscriptionsResponse {
    pub subscriptions: Vec<DigestSubscription>,
}
