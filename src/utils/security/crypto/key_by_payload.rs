use hex::encode;
use hmac::digest::InvalidLength;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum CipherKeyByError {
    #[error("{0}")]
    Serialization(#[from] serde_json::Error),

    #[error("{0}")]
    CreateFromSliceInvalidLength(#[from] InvalidLength),
}

pub struct CipherKeyByPayload(pub String);

pub fn create_key_by_payload<T: Serialize>(
    secret: &str,
    payload: &T,
) -> Result<CipherKeyByPayload, CipherKeyByError> {
    let json = serde_json::to_string(payload)?;

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())?;

    mac.update(json.as_bytes());

    Ok(CipherKeyByPayload(encode(mac.finalize().into_bytes())))
}
