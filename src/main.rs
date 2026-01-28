use crate::client::telegram::bot::TelegramBot;
use crate::client::telegram::client::TelegramHttpClient;
use std::sync::Arc;

mod client;
mod config;
mod server;

#[tokio::main]
async fn main() {
    let port = config::environment::ENV.get("APPLICATION_PORT");

    let bot: Arc<dyn TelegramBot> = Arc::new(TelegramHttpClient::new(
        config::environment::ENV.get("TELEGRAM_BOT_TOKEN"),
    ));

    server::http::run(port.as_str()).await;
}
