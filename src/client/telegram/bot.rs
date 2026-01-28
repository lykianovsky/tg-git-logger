use crate::client::telegram::client::TelegramHttpClient;
use crate::config::environment::ENV;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait TelegramBot: Send + Sync {
    async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<reqwest::Response, reqwest::Error>;
}

#[derive(Serialize, Debug)]
struct SendMessageRequest<'a> {
    chat_id: i64,
    parse_mode: &'a str,
    text: &'a str,
}

#[async_trait::async_trait]
impl TelegramBot for TelegramHttpClient {
    async fn send_message(
        &self,
        chat_id: i64,
        text: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let base = ENV.get("TELEGRAM_URL_BASE");
        let url = format!("{base}/bot{}/sendMessage", self.token);

        let request_body = SendMessageRequest {
            chat_id,
            text,
            parse_mode: "html",
        };

        self.client.post(&url).json(&request_body).send().await
    }
}
