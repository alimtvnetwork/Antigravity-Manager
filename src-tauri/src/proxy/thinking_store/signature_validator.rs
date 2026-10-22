//! Thought Block Signature Re-Validator.
//! Verifies server-side thinking signatures before injection to prevent malformed prompt payload errors.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureValidationResult {
    pub is_valid: bool,
    pub reason: Option<String>,
}

/// Validates whether a cached or received thinking block signature meets format invariants.
pub fn validate_thought_signature(signature: &str) -> SignatureValidationResult {
    let trimmed = signature.trim();

    if trimmed.is_empty() {
        return SignatureValidationResult {
            is_valid: false,
            reason: Some("Signature string is empty".to_string()),
        };
    }

    // Minimum cryptographic signature length (e.g. 16 chars)
    if trimmed.len() < 16 {
        return SignatureValidationResult {
            is_valid: false,
            reason: Some(format!("Signature too short (length {})", trimmed.len())),
        };
    }

    // Upper bound sanity check (no runaway buffers > 4KB)
    if trimmed.len() > 4096 {
        return SignatureValidationResult {
            is_valid: false,
            reason: Some("Signature exceeds maximum allowable length of 4096 bytes".to_string()),
        };
    }

    // Validate characters are standard Base64 / Hex or URL-safe ASCII
    let is_valid_charset = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '-' || c == '_');

    if !is_valid_charset {
        return SignatureValidationResult {
            is_valid: false,
            reason: Some("Signature contains invalid non-base64 characters".to_string()),
        };
    }

    SignatureValidationResult {
        is_valid: true,
        reason: None,
    }
}
