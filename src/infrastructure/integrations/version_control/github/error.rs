use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::cmp::PartialEq;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GithubGraphQLErrorType {
    NotFound,
    Forbidden,
    Unauthorized,
    RateLimited,
    ValidationError,
}

impl<'de> Deserialize<'de> for GithubGraphQLErrorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        GithubGraphQLErrorType::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl PartialEq<&str> for GithubGraphQLErrorType {
    fn eq(&self, other: &&str) -> bool {
        self.to_string() == *other
    }
}

impl fmt::Display for GithubGraphQLErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GithubGraphQLErrorType::NotFound => "NOT_FOUND",
            GithubGraphQLErrorType::Forbidden => "FORBIDDEN",
            GithubGraphQLErrorType::Unauthorized => "UNAUTHORIZED",
            GithubGraphQLErrorType::RateLimited => "RATE_LIMITED",
            GithubGraphQLErrorType::ValidationError => "VALIDATION_ERROR",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for GithubGraphQLErrorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "not_found" => Ok(GithubGraphQLErrorType::NotFound),
            "forbidden" => Ok(GithubGraphQLErrorType::Forbidden),
            "unauthorized" => Ok(GithubGraphQLErrorType::Unauthorized),
            "rate_limited" => Ok(GithubGraphQLErrorType::RateLimited),
            "validation_error" => Ok(GithubGraphQLErrorType::ValidationError),
            other => Err(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GithubGraphQLError {
    #[serde(rename = "type")]
    pub error_type: GithubGraphQLErrorType,
    pub message: String,
    pub path: Option<Vec<Value>>,
    #[serde(default)]
    pub extensions: Option<serde_json::Map<String, Value>>,
}

#[derive(Debug, Deserialize)]
pub struct GithubGraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GithubGraphQLError>>,
}
