#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "ru");

mod application;
mod bootstrap;
mod config;
mod delivery;
mod domain;
mod infrastructure;
mod utils;

#[tokio::main]
async fn main() {
    bootstrap::ApplicationBootstrap::new().run().await;
}
