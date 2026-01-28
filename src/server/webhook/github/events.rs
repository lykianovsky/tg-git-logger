use std::str::FromStr;

use crate::utils::notifier::message_builder::MessageBuilder;

pub trait GithubEvent {
    fn build(&self) -> MessageBuilder;
}

#[derive(Debug)]
pub enum GithubEventName {
    Ping,
    Push,
    PullRequest,
    Issues,
    Release,
    Workflow,
    Unknown(String),
}

impl FromStr for GithubEventName {
    type Err = ();

    fn from_str(external_string: &str) -> Result<Self, Self::Err> {
        match external_string {
            "push" => Ok(GithubEventName::Push),
            "ping" => Ok(GithubEventName::Ping),
            "pull_request" => Ok(GithubEventName::PullRequest),
            "issues" => Ok(GithubEventName::Issues),
            "release" => Ok(GithubEventName::Release),
            "workflow_run" => Ok(GithubEventName::Workflow),
            "workflow_job" => Ok(GithubEventName::Workflow),
            other => Ok(GithubEventName::Unknown(other.to_string())),
        }
    }
}
