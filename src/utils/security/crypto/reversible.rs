use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine as _, engine::general_purpose};
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReversibleCipherValue(String);

impl ReversibleCipherValue {
    pub fn new(value: String) -> Result<Self, CipherError> {
        // проверяем что это валидный base64
        let decoded = general_purpose::STANDARD.decode(&value)?;

        // минимум 12 байт nonce + хоть что-то
        if decoded.len() < 12 {
            return Err(CipherError::InvalidEncryptedData);
        }

        Ok(Self(value))
    }

    /// Создать без проверки — для внутреннего использования в encrypt()
    pub(crate) fn new_unchecked(value: String) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
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

    pub fn encrypt(&self, plaintext: &str) -> Result<ReversibleCipherValue, CipherError> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| CipherError::EncryptionFailed)?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(ReversibleCipherValue::new_unchecked(
            general_purpose::STANDARD.encode(result),
        ))
    }

    pub fn decrypt<T>(&self, encrypted: T) -> Result<String, CipherError>
    where
        T: AsRef<str>,
    {
        let encrypted_str = encrypted.as_ref();
        let data = general_purpose::STANDARD.decode(encrypted_str)?;

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

pub fn generate_secret_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    general_purpose::STANDARD.encode(key)
}
