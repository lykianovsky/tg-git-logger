pub trait StaticDomainEvent: Send + Sync + erased_serde::Serialize {
    fn event_name(&self) -> &'static str;
}

pub trait DomainEvent: StaticDomainEvent {
    const EVENT_NAME: &'static str;
}

impl<T: DomainEvent> StaticDomainEvent for T {
    fn event_name(&self) -> &'static str {
        T::EVENT_NAME
    }
}

erased_serde::serialize_trait_object!(StaticDomainEvent);
