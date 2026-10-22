//! Client Request Body Size Guard.
//! Inspects incoming client payload length against per-model and multimodal ceilings,
//! enforcing early HTTP 413 Payload Too Large rejection before memory exhaustion.

/// Default size ceilings in bytes.
pub const DEFAULT_TEXT_PAYLOAD_CEILING_BYTES: usize = 10 * 1024 * 1024; // 10 MB
pub const DEFAULT_MULTIMODAL_PAYLOAD_CEILING_BYTES: usize = 50 * 1024 * 1024; // 50 MB
pub const ABSOLUTE_HARD_LIMIT_BYTES: usize = 100 * 1024 * 1024; // 100 MB

/// Result of evaluating a request payload length.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeGuardOutcome {
    Accepted,
    RejectedTooLarge { actual_bytes: usize, ceiling_bytes: usize },
}

/// Evaluates content length against model capability thresholds.
pub fn evaluate_request_size(
    content_length: Option<usize>,
    actual_body_len: usize,
    is_multimodal: bool,
) -> SizeGuardOutcome {
    let effective_len = content_length.unwrap_or(actual_body_len).max(actual_body_len);

    let ceiling = if is_multimodal {
        DEFAULT_MULTIMODAL_PAYLOAD_CEILING_BYTES
    } else {
        DEFAULT_TEXT_PAYLOAD_CEILING_BYTES
    };

    if effective_len > ceiling {
        SizeGuardOutcome::RejectedTooLarge {
            actual_bytes: effective_len,
            ceiling_bytes: ceiling,
        }
    } else {
        SizeGuardOutcome::Accepted
    }
}

/// Helper returning true if the model name suggests multimodal capabilities.
pub fn is_multimodal_model_name(model: &str) -> bool {
    let lower = model.to_lowercase();
    lower.contains("vision")
        || lower.contains("flash")
        || lower.contains("claude-3")
        || lower.contains("gemini-1.5")
        || lower.contains("gemini-2.0")
        || lower.contains("gpt-4o")
}
