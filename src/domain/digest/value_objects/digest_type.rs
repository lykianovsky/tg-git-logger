use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DigestType {
    Daily,
    Weekly,
}

impl DigestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DigestType::Daily => "daily",
            DigestType::Weekly => "weekly",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "daily" => Some(DigestType::Daily),
            "weekly" => Some(DigestType::Weekly),
            _ => None,
        }
    }
}
