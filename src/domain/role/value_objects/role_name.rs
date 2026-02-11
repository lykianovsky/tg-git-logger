use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum RoleName {
    Admin,
    User,
}

impl fmt::Display for RoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RoleName::Admin => "admin",
            RoleName::User => "user",
        };
        write!(f, "{}", s)
    }
}

/// Конвертировать String из БД в enum
impl FromStr for RoleName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(RoleName::Admin),
            "user" => Ok(RoleName::User),
            _ => Err(format!("Unknown role: {}", s)),
        }
    }
}
