use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Deserializer, Serializer};
use sha2::Digest;

const LEGACY_FIXED_NONCE: &[u8; 12] = b"antigravsalt";
const ENCRYPTED_PREFIX: &str = "ag_enc_";
const ENCRYPTED_V2_PREFIX: &str = "ag_enc_v2_";
pub const ENCRYPTED_RSA_V1_PREFIX: &str = "ag_rsa_v1_";
const RSA_BLOCK_MAGIC: &[u8; 8] = b"AGM_RSA1";

/// Pure-Rust 2048-bit unsigned big integer (32 x u64 limbs, little-endian) for RSA-2048 modular exponentiation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BigUint2048(pub [u64; 32]);

impl BigUint2048 {
    pub const fn zero() -> Self {
        Self([0u64; 32])
    }

    pub const fn from_u64(v: u64) -> Self {
        let mut limbs = [0u64; 32];
        limbs[0] = v;
        Self(limbs)
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }

    pub fn from_be_bytes(bytes: &[u8]) -> Self {
        let mut padded = [0u8; 256];
        let take = bytes.len().min(256);
        let src_start = bytes.len() - take;
        padded[256 - take..].copy_from_slice(&bytes[src_start..]);
        let mut limbs = [0u64; 32];
        for i in 0..32 {
            let offset = (31 - i) * 8;
            let mut chunk = [0u8; 8];
            chunk.copy_from_slice(&padded[offset..offset + 8]);
            limbs[i] = u64::from_be_bytes(chunk);
        }
        Self(limbs)
    }

    pub fn to_be_bytes(&self) -> [u8; 256] {
        let mut out = [0u8; 256];
        for i in 0..32 {
            let offset = (31 - i) * 8;
            out[offset..offset + 8].copy_from_slice(&self.0[i].to_be_bytes());
        }
        out
    }

    pub fn to_hex(&self) -> String {
        let bytes = self.to_be_bytes();
        let mut s = String::with_capacity(512);
        for b in &bytes {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }

    pub fn from_hex(hex_str: &str) -> Result<Self, String> {
        let clean: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
        if clean.is_empty() || clean.len() > 512 {
            return Err("Invalid 2048-bit hex length".to_string());
        }
        let padded = format!("{:0>512}", clean);
        let mut bytes = [0u8; 256];
        for i in 0..256 {
            bytes[i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16)
                .map_err(|e| format!("Invalid hex byte: {}", e))?;
        }
        Ok(Self::from_be_bytes(&bytes))
    }

    pub fn bit(&self, idx: usize) -> bool {
        let word = idx / 64;
        let bit = idx % 64;
        if word >= 32 {
            false
        } else {
            ((self.0[word] >> bit) & 1) == 1
        }
    }

    pub fn highest_bit(&self) -> Option<usize> {
        for i in (0..32).rev() {
            if self.0[i] != 0 {
                return Some(i * 64 + (63 - self.0[i].leading_zeros() as usize));
            }
        }
        None
    }

    pub fn sub_u64(&self, rhs: u64) -> Self {
        let mut out = *self;
        let (v, mut borrow) = out.0[0].overflowing_sub(rhs);
        out.0[0] = v;
        for i in 1..32 {
            if !borrow {
                break;
            }
            let (nv, b) = out.0[i].overflowing_sub(1);
            out.0[i] = nv;
            borrow = b;
        }
        out
    }

    pub fn rem_u64(&self, divisor: u64) -> u64 {
        let mut rem: u128 = 0;
        let d = divisor as u128;
        for i in (0..32).rev() {
            rem = ((rem << 64) | (self.0[i] as u128)) % d;
        }
        rem as u64
    }

    /// Multiply by small scalar, add carry_in, and divide by small scalar
    pub fn mul_add_div_u64(&self, mul_by: u64, add_val: u64, div_by: u64) -> Self {
        let mut prod = [0u64; 33];
        let mut carry: u128 = add_val as u128;
        let m = mul_by as u128;
        for i in 0..32 {
            let v = (self.0[i] as u128) * m + carry;
            prod[i] = v as u64;
            carry = v >> 64;
        }
        prod[32] = carry as u64;

        let mut quot = [0u64; 32];
        let mut rem: u128 = 0;
        let d = div_by as u128;
        for i in (0..33).rev() {
            let cur = (rem << 64) | (prod[i] as u128);
            let q = (cur / d) as u64;
            rem = cur % d;
            if i < 32 {
                quot[i] = q;
            }
        }
        Self(quot)
    }

    pub fn mul_small_to_2048(a: &[u64; 16], b: &[u64; 16]) -> Self {
        let mut out = [0u64; 32];
        for i in 0..16 {
            let mut carry: u128 = 0;
            for j in 0..16 {
                let prod = (a[i] as u128) * (b[j] as u128) + (out[i + j] as u128) + carry;
                out[i + j] = prod as u64;
                carry = prod >> 64;
            }
            out[i + 16] = carry as u64;
        }
        Self(out)
    }

    /// Multiply two 2048-bit numbers modulo `modulus` (where `modulus < 2^2048`)
    pub fn mul_mod(&self, rhs: &Self, modulus: &Self) -> Self {
        let mut prod = [0u64; 64];
        for i in 0..32 {
            if self.0[i] == 0 {
                continue;
            }
            let mut carry: u128 = 0;
            let a = self.0[i] as u128;
            for j in 0..32 {
                let v = a * (rhs.0[j] as u128) + (prod[i + j] as u128) + carry;
                prod[i + j] = v as u64;
                carry = v >> 64;
            }
            prod[i + 32] = carry as u64;
        }

        let Some(mod_top_bit) = modulus.highest_bit() else {
            return Self::zero();
        };

        // Find highest set bit in `prod` (up to 4095)
        let mut top_bit_opt: Option<usize> = None;
        for w in (0..64).rev() {
            if prod[w] != 0 {
                top_bit_opt = Some(w * 64 + (63 - prod[w].leading_zeros() as usize));
                break;
            }
        }
        let Some(max_bit) = top_bit_opt else {
            return Self::zero();
        };

        if max_bit >= mod_top_bit {
            for shift in (0..=(max_bit - mod_top_bit)).rev() {
                let word_shift = shift / 64;
                let bit_shift = shift % 64;
                if prod_ge_shifted_mod(&prod, &modulus.0, word_shift, bit_shift) {
                    sub_shifted_mod(&mut prod, &modulus.0, word_shift, bit_shift);
                }
            }
        }

        let mut res = [0u64; 32];
        res.copy_from_slice(&prod[0..32]);
        Self(res)
    }

    /// Compute `self^exp mod modulus` via binary square-and-multiply
    pub fn mod_exp(&self, exp: &Self, modulus: &Self) -> Self {
        let Some(top_bit) = exp.highest_bit() else {
            return Self::from_u64(1);
        };
        let mut acc = *self;
        for bit_idx in (0..top_bit).rev() {
            acc = acc.mul_mod(&acc, modulus);
            if exp.bit(bit_idx) {
                acc = acc.mul_mod(self, modulus);
            }
        }
        acc
    }
}

fn get_shifted_mod_word(mod32: &[u64; 32], idx: usize, word_shift: usize, bit_shift: usize) -> u64 {
    if idx < word_shift {
        return 0;
    }
    let m_idx = idx - word_shift;
    let cur = if m_idx < 32 { mod32[m_idx] } else { 0 };
    if bit_shift == 0 {
        cur
    } else {
        let prev = if m_idx > 0 && m_idx - 1 < 32 {
            mod32[m_idx - 1]
        } else {
            0
        };
        (cur << bit_shift) | (prev >> (64 - bit_shift))
    }
}

fn prod_ge_shifted_mod(
    prod: &[u64; 64],
    mod32: &[u64; 32],
    word_shift: usize,
    bit_shift: usize,
) -> bool {
    let top_word = (word_shift + 33).min(63);
    for w in ((word_shift + 33)..64).rev() {
        if prod[w] != 0 {
            return true;
        }
    }
    for w in (word_shift..=top_word).rev() {
        let mw = get_shifted_mod_word(mod32, w, word_shift, bit_shift);
        if prod[w] != mw {
            return prod[w] > mw;
        }
    }
    true
}

fn sub_shifted_mod(prod: &mut [u64; 64], mod32: &[u64; 32], word_shift: usize, bit_shift: usize) {
    let top_word = (word_shift + 33).min(63);
    let mut borrow = false;
    for w in word_shift..=top_word {
        let mw = get_shifted_mod_word(mod32, w, word_shift, bit_shift);
        let (d1, b1) = prod[w].overflowing_sub(mw);
        let (d2, b2) = d1.overflowing_sub(u64::from(borrow));
        prod[w] = d2;
        borrow = b1 || b2;
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct RsaVaultKeypair {
    pub algorithm: String,
    pub key_bits: u32,
    pub n_hex: String,
    pub e_hex: String,
    pub d_hex: String,
}

/// Deterministic 2048-bit RSA parameters from RFC 3526 Group 2 safe prime P (1024-bit) and Sophie Germain prime Q (1023-bit)
fn compute_default_rsa_2048_keypair() -> (BigUint2048, BigUint2048, BigUint2048) {
    // RFC 3526 1024-bit MODP Group 2 safe prime P (little-endian u64 limbs)
    let p_limbs: [u64; 16] = [
        0xFFFFFFFFFFFFFFFF,
        0x49286651ECE65381,
        0xAE9F24117C4B1FE6,
        0xEE386BFB5A899FA5,
        0x0BFF5CB6F406B7ED,
        0xF44C42E9A637ED6B,
        0xE485B576625E7EC6,
        0x4FE1356D6D51C245,
        0x302B0A6DF25F1437,
        0xEF9519B3CD3A431B,
        0x514A08798E3404DD,
        0x020BBEA63B139B22,
        0x29024E088A67CC74,
        0xC4C6628B80DC1CD1,
        0xC90FDAA22168C234,
        0xFFFFFFFFFFFFFFFF,
    ];
    // Second distinct 1024-bit prime Q = (P - 1) / 2 (Sophie Germain prime of P)
    let mut q_limbs = [0u64; 16];
    for i in 0..16 {
        let hi_bit = if i < 15 { p_limbs[i + 1] << 63 } else { 0 };
        q_limbs[i] = (p_limbs[i] >> 1) | hi_bit;
    }

    let n = BigUint2048::mul_small_to_2048(&p_limbs, &q_limbs);
    let mut p_minus_1 = p_limbs;
    p_minus_1[0] -= 1;
    let mut q_minus_1 = q_limbs;
    q_minus_1[0] -= 1;
    let phi = BigUint2048::mul_small_to_2048(&p_minus_1, &q_minus_1);

    let e_val: u64 = 65537;
    let phi_rem = phi.rem_u64(e_val);
    let mut k_val: u64 = 1;
    for candidate in 1..e_val {
        if (candidate * phi_rem + 1) % e_val == 0 {
            k_val = candidate;
            break;
        }
    }
    let d = phi.mul_add_div_u64(k_val, 1, e_val);
    (n, BigUint2048::from_u64(e_val), d)
}

fn get_rsa_vault_key_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".agm").join("ssh").join("rsa_vault_key.json"))
}

/// Ensure `~/.agm/ssh/rsa_vault_key.json` exists and return `(n, e, d)`
pub fn load_or_init_rsa_2048_keypair() -> (BigUint2048, BigUint2048, BigUint2048) {
    let default_kp = compute_default_rsa_2048_keypair();
    let Some(vault_path) = get_rsa_vault_key_path() else {
        return default_kp;
    };

    if vault_path.exists() {
        if let Ok(raw) = std::fs::read_to_string(&vault_path) {
            if let Ok(stored) = serde_json::from_str::<RsaVaultKeypair>(&raw) {
                if let (Ok(n), Ok(e), Ok(d)) = (
                    BigUint2048::from_hex(&stored.n_hex),
                    BigUint2048::from_hex(&stored.e_hex),
                    BigUint2048::from_hex(&stored.d_hex),
                ) {
                    if !n.is_zero() && !e.is_zero() && !d.is_zero() {
                        return (n, e, d);
                    }
                }
            }
        }
    }

    if let Some(parent) = vault_path.parent() {
        // Justification: best-effort directory creation for RSA vault metadata; in-memory keypair still works
        crate::error::record_ignored(
            std::fs::create_dir_all(parent),
            "create ~/.agm/ssh vault dir",
        );
    }

    let stored = RsaVaultKeypair {
        algorithm: "RSA-2048".to_string(),
        key_bits: 2048,
        n_hex: default_kp.0.to_hex(),
        e_hex: default_kp.1.to_hex(),
        d_hex: default_kp.2.to_hex(),
    };
    if let Ok(serialized) = serde_json::to_string_pretty(&stored) {
        // Justification: best-effort persistence of RSA-2048 vault keypair JSON
        crate::error::record_ignored(
            std::fs::write(&vault_path, serialized),
            "write rsa_vault_key.json",
        );
    }

    default_kp
}

/// Encrypt an SSH password using RSA-2048 public-key modular exponentiation (`m^e mod n`) + AES-256-GCM envelope.
/// Output always starts with `ag_rsa_v1_` and never exposes the plaintext password.
pub fn encrypt_password_rsa(plain: &str) -> Result<String, String> {
    if plain.starts_with(ENCRYPTED_RSA_V1_PREFIX) {
        return Ok(plain.to_string());
    }

    let (n, e, _d) = load_or_init_rsa_2048_keypair();

    let mut sym_key = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    let mut random_pad = [0u8; 16];
    OsRng.fill_bytes(&mut sym_key);
    OsRng.fill_bytes(&mut nonce_bytes);
    OsRng.fill_bytes(&mut random_pad);

    // Construct 256-byte block `m < n` with top byte = 0x00, second byte = 0x02, magic, sym_key, nonce, and salt
    let mut block = [0u8; 256];
    block[0] = 0x00;
    block[1] = 0x02;
    block[2..10].copy_from_slice(RSA_BLOCK_MAGIC);
    block[10..42].copy_from_slice(&sym_key);
    block[42..54].copy_from_slice(&nonce_bytes);
    block[54..70].copy_from_slice(&random_pad);

    let m = BigUint2048::from_be_bytes(&block);
    let c = m.mod_exp(&e, &n);
    let c_bytes = c.to_be_bytes();

    let cipher = Aes256Gcm::new(&sym_key.into());
    let aes_ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plain.as_bytes())
        .map_err(|err| format!("RSA-2048 hybrid AES-GCM encryption failed: {}", err))?;

    let encoded_rsa = general_purpose::URL_SAFE_NO_PAD.encode(c_bytes);
    let encoded_aes = general_purpose::URL_SAFE_NO_PAD.encode(aes_ciphertext);

    Ok(format!(
        "{}{}.{}",
        ENCRYPTED_RSA_V1_PREFIX, encoded_rsa, encoded_aes
    ))
}

/// Decrypt an `ag_rsa_v1_` ciphertext using the RSA-2048 private key (`c^d mod n`) + AES-256-GCM envelope.
pub fn decrypt_password_rsa(cipher: &str) -> Result<String, String> {
    let trimmed = cipher.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let body = trimmed
        .strip_prefix(ENCRYPTED_RSA_V1_PREFIX)
        .ok_or_else(|| "Ciphertext does not start with ag_rsa_v1_ prefix".to_string())?;

    let (encoded_rsa, encoded_aes) = body
        .split_once('.')
        .ok_or_else(|| "Malformed ag_rsa_v1_ ciphertext envelope".to_string())?;

    let c_bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(encoded_rsa)
        .map_err(|e| format!("RSA block base64 decode failed: {}", e))?;
    if c_bytes.len() != 256 {
        return Err(format!(
            "Invalid RSA-2048 block byte length: {}",
            c_bytes.len()
        ));
    }

    let aes_ciphertext = general_purpose::URL_SAFE_NO_PAD
        .decode(encoded_aes)
        .map_err(|e| format!("AES payload base64 decode failed: {}", e))?;

    let (n, _e, d) = load_or_init_rsa_2048_keypair();
    let c = BigUint2048::from_be_bytes(&c_bytes);
    let m = c.mod_exp(&d, &n);
    let block = m.to_be_bytes();

    if &block[2..10] != RSA_BLOCK_MAGIC {
        return Err("RSA-2048 block magic header mismatch during decryption".to_string());
    }

    let mut sym_key = [0u8; 32];
    let mut nonce_bytes = [0u8; 12];
    sym_key.copy_from_slice(&block[10..42]);
    nonce_bytes.copy_from_slice(&block[42..54]);

    let aead = Aes256Gcm::new(&sym_key.into());
    let plain_bytes = aead
        .decrypt(Nonce::from_slice(&nonce_bytes), aes_ciphertext.as_ref())
        .map_err(|e| format!("RSA-2048 hybrid AES-GCM decryption failed: {}", e))?;

    String::from_utf8(plain_bytes)
        .map_err(|e| format!("Decrypted password is not valid UTF-8: {}", e))
}

fn get_encryption_key() -> [u8; 32] {
    let device_id = machine_uid::get().unwrap_or_else(|_| "default".to_string());
    let mut key = [0u8; 32];
    let hash = sha2::Sha256::digest(device_id.as_bytes());
    key.copy_from_slice(&hash);
    key
}

pub fn serialize_password<S>(password: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if password.starts_with(ENCRYPTED_PREFIX)
        || password.starts_with(ENCRYPTED_V2_PREFIX)
        || password.starts_with(ENCRYPTED_RSA_V1_PREFIX)
    {
        return serializer.serialize_str(password);
    }

    let encrypted = encrypt_string(password).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encrypted)
}

pub fn deserialize_password<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    if raw.is_empty() {
        return Ok(raw);
    }

    if raw.starts_with(ENCRYPTED_RSA_V1_PREFIX) {
        match decrypt_password_rsa(&raw) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => {
                tracing::warn!("RSA password decryption failed");
                Ok(raw)
            }
        }
    } else if raw.starts_with(ENCRYPTED_V2_PREFIX) {
        match decrypt_string_v2(&raw[ENCRYPTED_V2_PREFIX.len()..]) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => {
                tracing::warn!(
                    "password decryption failed, key likely changed (machine-id differs from when it was encrypted)"
                );
                Ok(raw)
            }
        }
    } else if raw.starts_with(ENCRYPTED_PREFIX) {
        match decrypt_legacy(&raw[ENCRYPTED_PREFIX.len()..]) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => {
                tracing::warn!(
                    "password decryption failed, key likely changed (machine-id differs from when it was encrypted)"
                );
                Ok(raw)
            }
        }
    } else {
        match decrypt_legacy(&raw) {
            Ok(plaintext) => Ok(plaintext),
            Err(_) => Ok(raw),
        }
    }
}

pub fn encrypt_string(password: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key.into());

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let encoded_nonce = general_purpose::STANDARD_NO_PAD.encode(nonce_bytes);
    let encoded_ciphertext = general_purpose::STANDARD_NO_PAD.encode(ciphertext);
    Ok(format!(
        "{}{}.{}",
        ENCRYPTED_V2_PREFIX, encoded_nonce, encoded_ciphertext
    ))
}

fn decrypt_legacy(encrypted_base64: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(LEGACY_FIXED_NONCE);

    let ciphertext = general_purpose::STANDARD
        .decode(encrypted_base64)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion failed: {}", e))
}

fn decrypt_string_v2(encrypted: &str) -> Result<String, String> {
    let (nonce_base64, ciphertext_base64) = encrypted
        .split_once('.')
        .ok_or_else(|| "Invalid encrypted payload".to_string())?;

    let nonce_bytes = general_purpose::STANDARD_NO_PAD
        .decode(nonce_base64)
        .map_err(|e| format!("Nonce decode failed: {}", e))?;
    if nonce_bytes.len() != 12 {
        return Err("Invalid nonce length".to_string());
    }

    let ciphertext = general_purpose::STANDARD_NO_PAD
        .decode(ciphertext_base64)
        .map_err(|e| format!("Ciphertext decode failed: {}", e))?;

    let key = get_encryption_key();
    let cipher = Aes256Gcm::new(&key.into());
    let plaintext = cipher
        .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 conversion failed: {}", e))
}

pub fn decrypt_string(encrypted: &str) -> Result<String, String> {
    if encrypted.starts_with(ENCRYPTED_RSA_V1_PREFIX) {
        decrypt_password_rsa(encrypted)
    } else if encrypted.starts_with(ENCRYPTED_V2_PREFIX) {
        decrypt_string_v2(&encrypted[ENCRYPTED_V2_PREFIX.len()..])
    } else if encrypted.starts_with(ENCRYPTED_PREFIX) {
        decrypt_legacy(&encrypted[ENCRYPTED_PREFIX.len()..])
    } else {
        decrypt_legacy(encrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_cycle() {
        let password = "my_secret_password";
        let encrypted = encrypt_string(password).unwrap();

        assert!(encrypted.starts_with(ENCRYPTED_V2_PREFIX));
        assert_ne!(password, encrypted);

        let decrypted = decrypt_string(&encrypted).unwrap();
        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_rsa_2048_password_encrypt_decrypt_roundtrip() {
        let plain = "SshRsaSecret!2026#NodeW3";
        let cipher_a = encrypt_password_rsa(plain).expect("RSA encrypt A");
        let cipher_b = encrypt_password_rsa(plain).expect("RSA encrypt B");

        assert!(cipher_a.starts_with(ENCRYPTED_RSA_V1_PREFIX));
        assert!(cipher_b.starts_with(ENCRYPTED_RSA_V1_PREFIX));
        assert_ne!(cipher_a, cipher_b);
        assert!(!cipher_a.contains(plain));

        let dec_a = decrypt_password_rsa(&cipher_a).expect("RSA decrypt A");
        let dec_b = decrypt_password_rsa(&cipher_b).expect("RSA decrypt B");
        assert_eq!(dec_a, plain);
        assert_eq!(dec_b, plain);
        assert_eq!(
            decrypt_string(&cipher_a).expect("decrypt_string RSA"),
            plain
        );
    }

    #[test]
    fn test_encrypt_uses_unique_nonce() {
        let password = "my_secret_password";
        let encrypted_a = encrypt_string(password).unwrap();
        let encrypted_b = encrypt_string(password).unwrap();

        assert_ne!(encrypted_a, encrypted_b);
        assert_eq!(decrypt_string(&encrypted_a).unwrap(), password);
        assert_eq!(decrypt_string(&encrypted_b).unwrap(), password);
    }

    #[test]
    fn test_legacy_compatibility() {
        let password = "legacy_password";
        let key = get_encryption_key();
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(LEGACY_FIXED_NONCE);
        let ciphertext = cipher.encrypt(nonce, password.as_bytes()).unwrap();
        let legacy_encrypted = general_purpose::STANDARD.encode(ciphertext);

        assert!(!legacy_encrypted.starts_with(ENCRYPTED_PREFIX));

        let decrypted = decrypt_string(&legacy_encrypted).unwrap();
        assert_eq!(password, decrypted);
    }

    #[derive(Deserialize)]
    struct PasswordContainer {
        #[serde(deserialize_with = "deserialize_password")]
        password: String,
    }

    #[test]
    fn test_deserialize_password_plaintext() {
        let json = r#"{"password": "plain_password_123"}"#;
        let parsed: PasswordContainer = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.password, "plain_password_123");
    }

    #[test]
    fn test_deserialize_password_valid_encrypted() {
        let encrypted = encrypt_string("secret_pass_456").unwrap();
        let json = format!(r#"{{"password": "{}"}}"#, encrypted);
        let parsed: PasswordContainer = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.password, "secret_pass_456");
    }

    #[test]
    fn test_deserialize_password_failed_decryption_returns_raw() {
        // Corrupted v2 payload: should return raw string as fallback
        let corrupted_v2 = "ag_enc_v2_invalidnonce.invalidciphertext";
        let json_v2 = format!(r#"{{"password": "{}"}}"#, corrupted_v2);
        let parsed_v2: PasswordContainer = serde_json::from_str(&json_v2).unwrap();
        assert_eq!(parsed_v2.password, corrupted_v2);

        // Corrupted legacy payload with ag_enc_ prefix: should return raw string as fallback
        let corrupted_legacy = "ag_enc_invalidbase64content==";
        let json_legacy = format!(r#"{{"password": "{}"}}"#, corrupted_legacy);
        let parsed_legacy: PasswordContainer = serde_json::from_str(&json_legacy).unwrap();
        assert_eq!(parsed_legacy.password, corrupted_legacy);
    }
}
