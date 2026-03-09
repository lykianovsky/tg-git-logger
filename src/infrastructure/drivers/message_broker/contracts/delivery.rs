use crate::infrastructure::drivers::message_broker::contracts::acknowledger::BrokerMessageAcknowledger;
use crate::infrastructure::drivers::message_broker::contracts::envelope::MessageBrokerEnvelope;

pub struct BrokerDelivery {
    pub envelope: MessageBrokerEnvelope<Vec<u8>>,
    acknowledger: Box<dyn BrokerMessageAcknowledger>,
}

impl BrokerDelivery {
    pub fn new(
        envelope: MessageBrokerEnvelope<Vec<u8>>,
        acknowledger: Box<dyn BrokerMessageAcknowledger>,
    ) -> Self {
        Self {
            envelope,
            acknowledger,
        }
    }

    pub async fn ack(self) {
        self.acknowledger.ack().await;
    }

    pub async fn nack(self, requeue: bool) {
        self.acknowledger.nack(requeue).await;
    }

    pub async fn reject(self, reason: &str) {
        self.acknowledger.reject(reason).await;
    }
}
