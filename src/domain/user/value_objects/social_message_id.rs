use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct SocialMessageId(pub i32);
