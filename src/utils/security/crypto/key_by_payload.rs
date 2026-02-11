use hex::encode;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub struct CipherKeyByPayload(pub String);

pub fn create_key_by_payload<T: Serialize>(secret: &str, payload: &T) -> CipherKeyByPayload {
    let json = serde_json::to_string(payload).expect("Failed to serialize payload");

    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(json.as_bytes());

    CipherKeyByPayload(encode(mac.finalize().into_bytes()))
}
