use std::str::FromStr;

#[derive(Debug)]
pub enum GithubEventName {
    Ping,
    Push,
    PullRequest,
    Issues,
    Release,
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
            other => Ok(GithubEventName::Unknown(other.to_string())),
        }
    }
}
