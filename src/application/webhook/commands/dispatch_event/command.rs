use crate::domain::webhook::events::WebhookEvent;

pub struct DispatchWebhookEventExecutorCommand {
    pub event: Box<dyn WebhookEvent>,
}
