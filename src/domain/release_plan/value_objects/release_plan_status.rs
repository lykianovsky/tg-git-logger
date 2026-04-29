#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleasePlanStatus {
    Planned,
    Cancelled,
    Done,
}

impl ReleasePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Cancelled => "cancelled",
            Self::Done => "done",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "planned" => Some(Self::Planned),
            "cancelled" => Some(Self::Cancelled),
            "done" => Some(Self::Done),
            _ => None,
        }
    }
}
