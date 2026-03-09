use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct MessageBrokerEnvelope<T> {
    pub name: String,
    pub payload: T,
}
