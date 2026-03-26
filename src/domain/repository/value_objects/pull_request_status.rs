use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PullRequestStatus {
    Open,
    Closed,
    Merged,
}

impl fmt::Display for PullRequestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PullRequestStatus::Open => "open",
            PullRequestStatus::Closed => "closed",
            PullRequestStatus::Merged => "merged",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for PullRequestStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "open" => Ok(PullRequestStatus::Open),
            "closed" => Ok(PullRequestStatus::Closed),
            "merged" => Ok(PullRequestStatus::Merged),
            _ => Err(format!("Unknown pull request status: {}", s)),
        }
    }
}
