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

    pub async fn requeue(self) {
        self.acknowledger.requeue().await;
    }

    pub async fn retry(self, reason: &str) {
        self.acknowledger.retry(reason).await;
    }

    pub async fn reject(self, reason: &str) {
        self.acknowledger.reject(reason).await;
    }
}
