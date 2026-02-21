use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum VersionControlType {
    Github,
}

impl fmt::Display for VersionControlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            VersionControlType::Github => "github",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for VersionControlType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(VersionControlType::Github),
            _ => Err(format!("Unknown version control type: {}", s)),
        }
    }
}
