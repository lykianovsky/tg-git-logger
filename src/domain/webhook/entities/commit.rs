use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookCommit {
    pub id: String,                // полный хеш
    pub short_id: String,          // первые 7 символов
    pub message: String,           // сообщение коммита
    pub author: String,            // автор коммита
    pub url: String,               // ссылка на коммит
    pub timestamp: Option<String>, // время коммита
}
