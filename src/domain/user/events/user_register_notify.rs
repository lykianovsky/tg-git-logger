use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::user::value_objects::social_chat_id::SocialChatId;
use crate::domain::user::value_objects::social_type::SocialType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserRegisterNotifyEvent {
    pub social_type: SocialType,
    pub social_chat_id: SocialChatId,
}

impl StaticDomainEvent for UserRegisterNotifyEvent {
    const EVENT_NAME: &'static str = "webhook.push";
}
