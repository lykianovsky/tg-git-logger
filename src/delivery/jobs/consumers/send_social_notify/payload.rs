use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind, MessageBrokerMessageKindJobPriority,
};
use crate::utils::builder::message::MessageBuilder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SendSocialNotifyJob {
    pub social_type: SocialType,
    pub chat_id: SocialChatId,
    pub message: MessageBuilder,
}

impl SendSocialNotifyJob {
    pub const NAME: &'static str = "send_social_notify";
}

impl MessageBrokerMessage for SendSocialNotifyJob {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Job(MessageBrokerMessageKindJobPriority::Critical)
    }
}
