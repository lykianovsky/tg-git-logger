use reqwest::Client;

pub struct TelegramHttpClient {
    pub token: String,
    pub client: Client,
}

impl TelegramHttpClient {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            client: Client::new(),
        }
    }
}
