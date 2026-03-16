use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum RoleName {
    Admin,
    Developer,
    QualityAssurance,
}

impl RoleName {
    pub(crate) fn to_str(&self) -> &'static str {
        match self {
            RoleName::Admin => "admin",
            RoleName::Developer => "developer",
            RoleName::QualityAssurance => "quality_assurance",
        }
    }
}

impl fmt::Display for RoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoleName::Admin => write!(f, "{}", self.to_str()),
            RoleName::Developer => write!(f, "{}", self.to_str()),
            RoleName::QualityAssurance => write!(f, "{}", self.to_str()),
        }
    }
}

impl FromStr for RoleName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(RoleName::Admin),
            "developer" => Ok(RoleName::Developer),
            "quality_assurance" => Ok(RoleName::QualityAssurance),
            _ => Err(format!("Unknown role name: {}", s)),
        }
    }
}
