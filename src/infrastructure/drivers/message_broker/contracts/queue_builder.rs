use crate::infrastructure::drivers::message_broker::contracts::delivery::BrokerDelivery;
use crate::infrastructure::drivers::message_broker::contracts::queue::MessageBrokerQueue;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;

pub type MessageBrokerStream<'a> = Pin<Box<dyn Stream<Item = BrokerDelivery> + Send + 'a>>;

pub struct MessageBrokerQueuesBuilder {
    queues: Vec<Arc<MessageBrokerQueue>>,
}

impl MessageBrokerQueuesBuilder {
    pub fn new_with_capacity(capacity: usize) -> Self {
        Self {
            queues: Vec::with_capacity(capacity),
        }
    }

    pub fn bind(mut self, queue: Arc<MessageBrokerQueue>) -> Self {
        self.queues.push(queue);

        self
    }

    pub fn build(self) -> Vec<Arc<MessageBrokerQueue>> {
        self.queues
    }
}
