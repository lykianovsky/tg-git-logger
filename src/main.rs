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
