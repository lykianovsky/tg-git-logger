use crate::domain::shared::events::event::DomainEvent;
use crate::domain::user::entities::user::User;
use crate::domain::user::entities::user_social_account::UserSocialAccount;
use crate::domain::user::entities::user_vc_account::UserVersionControlAccount;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use crate::infrastructure::drivers::message_broker::contracts::publisher::{
    MessageBrokerMessage, MessageBrokerMessageKind,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserRegistrationSuccessEvent {
    pub social_type: SocialType,
    pub chat_id: SocialChatId,

    pub user: User,
    pub user_social_account: UserSocialAccount,
    pub user_version_control_account: UserVersionControlAccount,
}

impl DomainEvent for UserRegistrationSuccessEvent {
    const EVENT_NAME: &'static str = "user.registration.success";
}

impl MessageBrokerMessage for UserRegistrationSuccessEvent {
    fn name(&self) -> &'static str {
        Self::EVENT_NAME
    }

    fn kind(&self) -> MessageBrokerMessageKind {
        MessageBrokerMessageKind::Event
    }
}
