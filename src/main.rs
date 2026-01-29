mod client;
mod config;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    let port = config::environment::ENV.get("APPLICATION_PORT");

    server::telegram_bot::run();
    server::http::run(port.as_str()).await;
}
