pub trait DomainEvent: Send + Sync + erased_serde::Serialize {
    fn event_name(&self) -> &'static str;
}

pub trait StaticDomainEvent: DomainEvent {
    const EVENT_NAME: &'static str;
}

impl<T: StaticDomainEvent> DomainEvent for T {
    fn event_name(&self) -> &'static str {
        T::EVENT_NAME
    }
}

erased_serde::serialize_trait_object!(DomainEvent);
