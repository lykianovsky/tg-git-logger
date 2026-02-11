pub mod consumer;
pub mod consumer_runner;
pub mod event;
pub mod publisher;
pub mod retry_policy;

pub const EXCHANGE_NAME: &str = "domain_events";
pub const EXCHANGE_KIND: lapin::ExchangeKind = lapin::ExchangeKind::Topic;
