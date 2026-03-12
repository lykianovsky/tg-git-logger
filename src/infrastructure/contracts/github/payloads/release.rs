use crate::domain::webhook::events::release::WebhookReleaseEvent;
use crate::infrastructure::contracts::github::event_type::GithubEvent;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct GithubReleaseEvent {
    pub action: String, // "published", "created", "edited", "deleted", "prereleased"
    pub release: GithubRelease,
    pub repository: GithubRepository,
    pub sender: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    pub id: u64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: String,
    pub published_at: Option<String>,
    pub html_url: String,
    pub author: GithubUser,
}

#[derive(Debug, Deserialize)]
pub struct GithubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub avatar_url: Option<String>,
    pub id: u64,
}

impl GithubEvent for GithubReleaseEvent {
    type WebhookEvent = WebhookReleaseEvent;

    fn from_value(value: Value) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_value(value)
    }

    fn to_webhook_event(&self) -> Self::WebhookEvent {
        WebhookReleaseEvent {
            id: self.release.id,
            tag_name: self.release.tag_name.clone(),
            target_commitish: self.release.tag_name.clone(), // GitHub обычно не присылает target_commit, можно временно использовать тег
            name: self.release.name.clone(),
            body: self.release.body.clone(),
            draft: self.release.draft,
            prerelease: self.release.prerelease,
            created_at: Some(self.release.created_at.clone()),
            published_at: self.release.published_at.clone(),
            html_url: Some(self.release.html_url.clone()),
            author: Some(self.release.author.login.clone()),
            repo: self.repository.full_name.clone(),
            repo_url: self.repository.html_url.clone(),
        }
    }
}
