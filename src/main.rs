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
    if let Err(error) = bootstrap::ApplicationBootstrap::new().run().await {
        tracing::error!(error = %error, "Application boostrap was run with error")
    }
}
