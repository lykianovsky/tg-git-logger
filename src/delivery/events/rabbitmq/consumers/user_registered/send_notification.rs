use crate::domain::shared::events::consumer::{EventConsumer, EventConsumerError};
use crate::domain::shared::events::event::StaticDomainEvent;
use crate::domain::user::events::user_register_notify::UserRegisterNotifyEvent;
use async_trait::async_trait;

pub struct RabbitMQUserRegisteredSendNotificationConsumer {}

#[async_trait]
impl EventConsumer for RabbitMQUserRegisteredSendNotificationConsumer {
    type EventPayload = UserRegisterNotifyEvent;

    fn routing_key(&self) -> &'static str {
        <Self::EventPayload as StaticDomainEvent>::EVENT_NAME
    }

    fn queue_name(&self) -> &'static str {
        "tg-bot-logger.notification.send"
    }

    async fn handle(&self, payload: UserRegisterNotifyEvent) -> Result<(), EventConsumerError> {
        println!(
            "TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST TEST",
        );
        println!("{:?}", payload.social_chat_id);
        Ok(())
    }
}
