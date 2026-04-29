use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationEventKind {
    Pr,
    Review,
    Comment,
    Ci,
    Release,
}

impl NotificationEventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pr => "pr",
            Self::Review => "review",
            Self::Comment => "comment",
            Self::Ci => "ci",
            Self::Release => "release",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pr" => Some(Self::Pr),
            "review" => Some(Self::Review),
            "comment" => Some(Self::Comment),
            "ci" => Some(Self::Ci),
            "release" => Some(Self::Release),
            _ => None,
        }
    }

    pub fn all_default_enabled() -> Vec<Self> {
        vec![
            Self::Pr,
            Self::Review,
            Self::Comment,
            Self::Ci,
            Self::Release,
        ]
    }
}
