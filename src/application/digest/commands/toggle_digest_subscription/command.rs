use crate::domain::digest::value_objects::digest_subscription_id::DigestSubscriptionId;

pub struct ToggleDigestSubscriptionCommand {
    pub id: DigestSubscriptionId,
}
