use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CipherError {
    #[error("Invalid base64 encoding: {0}")]
    InvalidBase64(#[from] base64::DecodeError),

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed: data may be corrupted or tampered with")]
    DecryptionFailed,

    #[error("Invalid encrypted data: too short (minimum 12 bytes required)")]
    InvalidEncryptedData,

    #[error("Invalid nonce length: expected 12 bytes")]
    InvalidNonceLength,

    #[error("Invalid UTF-8 in decrypted data")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

pub struct ReversibleCipher {
    cipher: Aes256Gcm,
}

impl ReversibleCipher {
    /// Creates cipher from secret_key (base64 string)
    /// Panics if secret_key is invalid or not 32 bytes
    pub fn new(secret_key: &str) -> Self {
        let key_bytes = general_purpose::STANDARD
            .decode(secret_key)
            .expect("SECRET_KEY must be valid base64");

        assert_eq!(
            key_bytes.len(),
            32,
            "SECRET_KEY must be exactly 32 bytes (256 bits), got {} bytes",
            key_bytes.len()
        );

        let key: &[u8; 32] = key_bytes
            .as_slice()
            .try_into()
            .expect("Failed to convert key to [u8; 32]");

        let cipher = Aes256Gcm::new(key.into());

        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<String, CipherError> {
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| CipherError::EncryptionFailed)?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(result))
    }

    pub fn decrypt(&self, encrypted: &str) -> Result<String, CipherError> {
        let data = general_purpose::STANDARD.decode(encrypted)?;

        if data.len() < 12 {
            return Err(CipherError::InvalidEncryptedData);
        }

        let (nonce_bytes, ciphertext) = data.split_at(12);

        let nonce_array: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| CipherError::InvalidNonceLength)?;
        let nonce = Nonce::from(nonce_array);

        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CipherError::DecryptionFailed)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

/// Utility to generate a new secret key (run once)
pub fn generate_secret_key() -> String {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    general_purpose::STANDARD.encode(key)
}
