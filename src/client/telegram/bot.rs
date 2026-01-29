use crate::client::telegram::client::TelegramHttpClient;
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
        let url = format!("{}/bot{}/sendMessage", self.base, self.token);

        let request_body = SendMessageRequest {
            chat_id,
            text,
            parse_mode: "html",
        };

        self.client.post(&url).json(&request_body).send().await
    }
}
