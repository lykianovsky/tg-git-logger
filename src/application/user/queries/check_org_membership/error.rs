use crate::utils::security::crypto::reversible::CipherError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckOrgMembershipError {
    #[error("Database error: {0}")]
    DbError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Cipher error: {0}")]
    CipherError(#[from] CipherError),

    #[error("Membership check failed: {0}")]
    CheckFailed(String),
}
