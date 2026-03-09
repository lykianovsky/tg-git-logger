use crate::domain::shared::events::event::DomainEvent;
use crate::domain::shared::events::event_listener::EventListener;
use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("Error for extract registry raw parser by event_name: {0}")]
    DispatchRawRegistryParser(String),

    #[error("Error deserialize received raw payload: {0}")]
    Deserialize(String),
}

// Trait для десериализации
#[async_trait]
trait EventDeserializer: Send + Sync {
    async fn deserialize_and_dispatch(
        &self,
        payload: &[u8],
        event_bus: &EventBus,
    ) -> Result<(), EventBusError>;
}

// Обработчик конкретного типа
struct TypedEventHandler<E: DomainEvent + 'static> {
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E> EventDeserializer for TypedEventHandler<E>
where
    E: DomainEvent + serde::de::DeserializeOwned + 'static,
{
    async fn deserialize_and_dispatch(
        &self,
        payload: &[u8],
        event_bus: &EventBus,
    ) -> Result<(), EventBusError> {
        let event: E = serde_json::from_slice(payload)
            .map_err(|e| EventBusError::Deserialize(e.to_string()))?;
        event_bus.dispatch(&event).await;
        Ok(())
    }
}

struct ListenerWrapper<E: DomainEvent> {
    listener: Arc<dyn EventListener<E> + Send + Sync>,
}

pub struct EventBus {
    listeners: Mutex<HashMap<TypeId, Vec<Box<dyn Any + Send + Sync>>>>,
    registry: Mutex<HashMap<String, Arc<dyn EventDeserializer>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Mutex::new(HashMap::new()),
            registry: Mutex::new(HashMap::new()),
        }
    }

    pub async fn on<
        E: DomainEvent + serde::de::DeserializeOwned + 'static,
        L: EventListener<E> + 'static,
    >(
        &self,
        listener: L,
    ) {
        {
            let mut map = self.listeners.lock().await;
            let entry = map.entry(TypeId::of::<E>()).or_insert_with(Vec::new);
            let value = Box::new(ListenerWrapper {
                listener: Arc::new(listener),
            });
            entry.push(value);
        }

        {
            let mut registry = self.registry.lock().await;
            if !registry.contains_key(E::EVENT_NAME) {
                let handler = Arc::new(TypedEventHandler::<E> {
                    _phantom: std::marker::PhantomData,
                });
                registry.insert(E::EVENT_NAME.to_string(), handler);
            }
        }
    }

    pub async fn dispatch<E: DomainEvent + 'static>(&self, event: &E) {
        let listeners: Vec<Arc<dyn EventListener<E> + Send + Sync>> = {
            let map = self.listeners.lock().await;
            map.get(&TypeId::of::<E>())
                .map(|v| {
                    v.iter()
                        .map(|w| {
                            w.downcast_ref::<ListenerWrapper<E>>()
                                .unwrap()
                                .listener
                                .clone()
                        })
                        .collect()
                })
                .unwrap_or_default()
        };

        for listener in listeners {
            listener.handle(event).await;
        }
    }

    pub async fn dispatch_raw(
        &self,
        event_name: &str,
        payload: &[u8],
    ) -> Result<(), EventBusError> {
        let handler = {
            let registry = self.registry.lock().await;

            registry
                .get(event_name)
                .ok_or_else(|| format!("Unknown event: {}", event_name))
                .map_err(|e| EventBusError::DispatchRawRegistryParser(e.to_string()))?
                .clone()
        };

        handler.deserialize_and_dispatch(payload, self).await
    }
}
