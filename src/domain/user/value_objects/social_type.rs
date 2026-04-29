use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum SocialType {
    Telegram,
}

impl fmt::Display for SocialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SocialType::Telegram => "telegram",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for SocialType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "telegram" => Ok(SocialType::Telegram),
            _ => Err(format!("Unknown social type: {}", s)),
        }
    }
}
