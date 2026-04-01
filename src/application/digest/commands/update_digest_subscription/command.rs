use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;

pub struct UpdateDigestSubscriptionCommand {
    pub id: DigestSubscriptionId,
    pub is_active: Option<bool>,
    pub send_hour: Option<i8>,
    pub send_minute: Option<i8>,
}
