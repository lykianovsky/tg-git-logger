use crate::applications::state::ApplicationState;
use crate::config::environment::ENV;
use crate::infrastructure::database;
use std::sync::Arc;

mod applications;
mod config;
mod client;
pub mod domain;
mod infrastructure;


#[tokio::main]
async fn main() {
    let is_debug_enabled: bool = ENV.get("DEBUG").parse().unwrap();

    let debug_filter: &str = if is_debug_enabled { "debug" } else { "info" };

    tracing_subscriber::fmt()
        .with_env_filter(debug_filter)
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::ENTER
                | tracing_subscriber::fmt::format::FmtSpan::EXIT,
        )
        .with_target(true)
        .with_file(true)
        .with_line_number(is_debug_enabled)
        .init();


    let db = database::mysql::connect(ENV.get("DATABASE_URL")).await;

    let application_state = Arc::new(ApplicationState {
        db
    });

    let http_handle = tokio::spawn(async move {
        applications::telegram::bot::run().await;
    });

    let telegram_bot_handle = tokio::spawn(async move {
        applications::http::run(application_state).await;
    });

    tokio::try_join!(http_handle, telegram_bot_handle).unwrap();
}
