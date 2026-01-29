use reqwest::Client;

pub struct TelegramHttpClient {
    pub base: String,
    pub token: String,
    pub client: Client,
}

impl TelegramHttpClient {
    pub fn new(base: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            token: token.into(),
            client: Client::new(),
        }
    }
}
