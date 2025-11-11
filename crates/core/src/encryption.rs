use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::{rngs::OsRng, RngCore};
use zeroize::Zeroizing;

pub trait KeyProvider: Send + Sync {
    fn key_material(&self) -> Result<KeyMaterial>;
}

#[derive(Debug)]
pub struct KeyMaterial {
    bytes: Zeroizing<Vec<u8>>,
}

impl Clone for KeyMaterial {
    fn clone(&self) -> Self {
        Self {
            bytes: Zeroizing::new(self.bytes.to_vec()),
        }
    }
}

impl KeyMaterial {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        if bytes.len() < 32 {
            return Err(anyhow!("Key material must be at least 32 bytes"));
        }
        Ok(Self {
            bytes: Zeroizing::new(bytes),
        })
    }

    pub fn to_key_bytes(&self) -> Result<[u8; 32]> {
        let mut key = [0u8; 32];
        key.copy_from_slice(&self.bytes[..32]);
        Ok(key)
    }
}

pub struct EnvelopeEncryption<P: KeyProvider + 'static> {
    key_provider: Arc<P>,
}

impl<P: KeyProvider + 'static> EnvelopeEncryption<P> {
    pub fn new(key_provider: Arc<P>) -> Self {
        Self { key_provider }
    }

    pub fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key_bytes = self.key_provider.key_material()?.to_key_bytes()?;
        let key = Key::from(key_bytes);
        let cipher = ChaCha20Poly1305::new(&key);
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);
        let mut ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("encryption failed: {e}"))?;

        let mut output = nonce_bytes.to_vec();
        output.append(&mut ciphertext);
        Ok(output)
    }

    pub fn open(&self, payload: &[u8]) -> Result<Vec<u8>> {
        if payload.len() < 13 {
            return Err(anyhow!("ciphertext too short"));
        }

        let (nonce_bytes, ciphertext) = payload.split_at(12);
        let key_bytes = self.key_provider.key_material()?.to_key_bytes()?;
        let key = Key::from(key_bytes);
        let cipher = ChaCha20Poly1305::new(&key);
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);
        let nonce = Nonce::from(nonce_arr);

        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("decryption failed: {e}"))
    }
}

pub struct EnvKeyProvider {
    env_key: String,
}

impl EnvKeyProvider {
    pub fn from_env(var: &str) -> Result<Self> {
        let value = std::env::var(var)
            .map_err(|_| anyhow!("Environment variable {var} missing for encryption key"))?;
        Ok(Self { env_key: value })
    }
}

impl KeyProvider for EnvKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        KeyMaterial::new(hex::decode(&self.env_key).context("Invalid hex in env key")?)
    }
}

pub struct StaticKeyProvider {
    key: KeyMaterial,
}

impl StaticKeyProvider {
    pub fn new(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            key: KeyMaterial::new(bytes)?,
        })
    }
}

impl KeyProvider for StaticKeyProvider {
    fn key_material(&self) -> Result<KeyMaterial> {
        Ok(self.key.clone())
    }
}
