use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::prelude::*;
use rand::RngCore;
use sha2::{Digest, Sha256};

use super::*;

// ---------------------------------------------------------------------------
// Cryptographic Secret Encryption & SSH RSA Key Derivation
// ---------------------------------------------------------------------------

/// Derive encryption key from machine identity and salt
pub(crate) fn derive_aes_key(salt: &str) -> [u8; 32] {
    let machine_id = machine_uid::get().unwrap_or_else(|_| "antigravity-default-seed".to_string());
    let mut hasher = Sha256::new();
    hasher.update(machine_id.as_bytes());
    hasher.update(b":vault-kdf:");
    hasher.update(salt.as_bytes());
    let hash = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&hash[0..32]);
    key
}

/// Compute SSH RSA public key token and fingerprint
pub fn compute_ssh_rsa_identity(salt: &str, secret: &str) -> (String, String) {
    let mut hasher = Sha256::new();
    hasher.update(b"ssh-rsa-vault-token-v1:");
    hasher.update(salt.as_bytes());
    hasher.update(secret.as_bytes());
    let hash = hasher.finalize();

    let fingerprint = format!("SHA256:{}", BASE64_STANDARD.encode(hash));
    let pub_key = format!(
        "ssh-rsa AAAAB3NzaC1yc2E{} antigravity@node",
        BASE64_STANDARD.encode(hash)
    );
    (fingerprint, pub_key)
}

/// Encrypt secret using AES-256-GCM and generate SSH RSA token
pub fn encrypt_secret(secret: &str, salt: &str) -> Result<(String, String, String), String> {
    let key_bytes = derive_aes_key(salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to init cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, secret.as_bytes())
        .map_err(|e| format!("Failed to encrypt secret: {}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    let enc_b64 = BASE64_STANDARD.encode(&combined);
    let (fingerprint, pub_key) = compute_ssh_rsa_identity(salt, secret);

    Ok((enc_b64, fingerprint, pub_key))
}

/// Decrypt secret using AES-256-GCM
pub fn decrypt_secret(encrypted_b64: &str, salt: &str) -> Result<String, String> {
    let data = BASE64_STANDARD
        .decode(encrypted_b64)
        .map_err(|e| format!("Invalid base64 payload: {}", e))?;

    if data.len() < 12 {
        return Err("Payload too short for nonce".to_string());
    }

    let (nonce_bytes, ciphertext) = data.split_at(12);
    let key_bytes = derive_aes_key(salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| format!("Failed to init cipher: {}", e))?;

    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("Invalid utf8 secret: {}", e))
}
