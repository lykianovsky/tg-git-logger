use crate::domain::webhook::entities::commit::WebhookCommit;
use crate::domain::webhook::events::push::WebhookPushEvent;
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubPushEvent {
    #[serde(rename = "ref")]
    pub ref_field: String, // "refs/heads/main"
    pub before: String,
    pub after: String,
    pub compare: Option<String>, // ссылка на сравнение, может быть пустой
    pub created: Option<bool>,   // иногда отсутствует
    pub deleted: Option<bool>,
    pub forced: Option<bool>,

    pub head_commit: Option<GithubCommit>,

    pub commits: Vec<GithubCommit>,

    pub repository: GithubRepository,
    pub pusher: GithubPusher,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepository {
    pub id: u64,
    pub node_id: Option<String>,
    pub name: String,
    pub full_name: String,
    pub private: bool,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubPusher {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub id: u64,
    pub node_id: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubCommit {
    pub id: String,
    pub tree_id: Option<String>,
    pub distinct: Option<bool>,
    pub message: String,
    pub timestamp: Option<String>,
    pub url: String, // required
    pub added: Option<Vec<String>>,
    pub removed: Option<Vec<String>>,
    pub modified: Option<Vec<String>>,

    pub author: Option<CommitAuthor>,
    pub committer: Option<CommitAuthor>,
}

#[derive(Debug, Deserialize)]
pub struct CommitAuthor {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl GithubPushEvent {
    pub fn from_value(value: Value) -> Result<Self, String> {
        serde_json::from_value(value).map_err(|e| format!("Failed to parse push event: {}", e))
    }
}

impl GithubEvent for GithubPushEvent {
    type WebhookEvent = WebhookPushEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        // Мапим коммиты
        let commits = self
            .commits
            .iter()
            .map(|c| WebhookCommit {
                id: c.id.clone(),
                short_id: c.id.chars().take(7).collect(),
                message: c.message.clone(),
                author: c
                    .author
                    .as_ref()
                    .and_then(|a| a.name.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                url: c.url.clone(),
                timestamp: c.timestamp.clone(),
            })
            .collect();

        let branch = self
            .ref_field
            .strip_prefix("refs/heads/")
            .unwrap_or(&self.ref_field);

        WebhookPushEvent {
            source: self.pusher.name.clone(),
            repo: self.repository.full_name.clone(),
            repo_url: self.repository.html_url.clone(),
            ref_field: branch.to_string(),
            before: self.before.clone(),
            after: self.after.clone(),
            compare_url: self.compare.clone(),
            created: self.created.unwrap_or(false),
            deleted: self.deleted.unwrap_or(false),
            forced: self.forced.unwrap_or(false),
            commits,
        }
    }
}
