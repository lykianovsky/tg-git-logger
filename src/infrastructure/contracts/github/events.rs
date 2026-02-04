use crate::utils::builder::message::MessageBuilder;
use std::str::FromStr;

pub trait GithubEvent {
    fn build(&self) -> MessageBuilder;
}

#[derive(Debug)]
pub enum GithubEventType {
    Ping,
    Push,
    PullRequest,
    Issues,
    Release,
    Workflow,
    Unknown(String),
}

impl FromStr for GithubEventType {
    type Err = ();

    fn from_str(external_string: &str) -> Result<Self, Self::Err> {
        match external_string {
            "push" => Ok(GithubEventType::Push),
            "ping" => Ok(GithubEventType::Ping),
            "pull_request" => Ok(GithubEventType::PullRequest),
            "issues" => Ok(GithubEventType::Issues),
            "release" => Ok(GithubEventType::Release),
            "workflow_run" => Ok(GithubEventType::Workflow),
            other => Ok(GithubEventType::Unknown(other.to_string())),
        }
    }
}