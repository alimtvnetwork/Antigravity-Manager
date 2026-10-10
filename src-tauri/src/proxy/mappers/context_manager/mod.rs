//! Context Manager: token estimation and history purification.
//! Facade: implementation lives in submodules, each <= 500 lines.

//! Context Manager Module
//!
//! Responsible for estimating token usage and purifying context (stripping thinking blocks)
//! to prevent "Prompt is too long" errors and avoid invalid signatures.

use super::caveman_cleaner::CavemanCleaner;
use super::claude::models::{ClaudeRequest, ContentBlock, Message, MessageContent, SystemPrompt};
use super::openai::models::{OpenAIMessage, OpenAIRequest};
use super::rtk_cleaner::RtkCleaner;
use serde_json::{json, Value};
use tracing::{debug, info};

/// Helper to estimate tokens from text with multi-language awareness
///
/// Improved estimation algorithm:
/// - ASCII/English: ~4 characters per token
/// - Unicode/CJK: ~1.5 characters per token (Chinese, Japanese, Korean are tokenized differently)
/// - Adds 15% safety margin to prevent underestimation
pub(crate) fn estimate_tokens_from_str(s: &str) -> u32 {
    if s.is_empty() {
        return 0;
    }

    let mut ascii_chars = 0u32;
    let mut unicode_chars = 0u32;

    for c in s.chars() {
        if c.is_ascii() {
            ascii_chars += 1;
        } else {
            unicode_chars += 1;
        }
    }

    // ASCII: ~4 chars/token, Unicode/CJK: ~1.5 chars/token
    let ascii_tokens = (ascii_chars as f32 / 4.0).ceil() as u32;
    let unicode_tokens = (unicode_chars as f32 / 1.5).ceil() as u32;

    // Add 15% safety margin to account for tokenizer variations
    ((ascii_tokens + unicode_tokens) as f32 * 1.15).ceil() as u32
}

/// Estimate token cost for an image from an OpenAI-format image_url.
/// Handles both base64 data URLs and remote URLs.
/// Gemini counts standard images at ~258 tokens. For very large images
/// (>1MB base64 payload), we scale up since high-res images tokenize higher.
fn estimate_image_tokens_from_url(url: &str) -> u32 {
    const BASE_IMAGE_TOKENS: u32 = 258;

    if url.starts_with("data:") {
        // data:image/png;base64,<data>
        // Extract the base64 portion after the comma
        if let Some(comma_pos) = url.find(',') {
            let base64_len = url.len() - comma_pos - 1;
            // Approximate raw bytes: base64 encodes 3 bytes into 4 chars
            let raw_bytes = (base64_len * 3) / 4;

            // Gemini: standard images = 258 tokens
            // High-res images (>1MB) scale higher, up to ~10k tokens for very large ones
            if raw_bytes > 4_000_000 {
                // >4MB: very high resolution
                10_000
            } else if raw_bytes > 1_000_000 {
                // 1-4MB: high resolution, scale linearly
                let factor = raw_bytes as f32 / 1_000_000.0;
                (BASE_IMAGE_TOKENS as f32 * factor * 4.0).ceil() as u32
            } else {
                BASE_IMAGE_TOKENS
            }
        } else {
            BASE_IMAGE_TOKENS
        }
    } else {
        // Remote URL: assume standard resolution
        BASE_IMAGE_TOKENS
    }
}

/// Estimate token cost for audio from a URL.
/// Audio is tokenized at roughly 32 tokens per second.
/// From base64, we estimate duration from payload size.
pub(crate) fn estimate_media_tokens_from_url(url: &str) -> u32 {
    const BASE_AUDIO_TOKENS: u32 = 500; // ~15 seconds default

    if url.starts_with("data:") {
        if let Some(comma_pos) = url.find(',') {
            let base64_len = url.len() - comma_pos - 1;
            let raw_bytes = (base64_len * 3) / 4;
            // Rough estimate: 16kHz, 16-bit mono audio ≈ 32KB/s
            // 32 tokens/second
            let estimated_seconds = raw_bytes as f32 / 32_000.0;
            let tokens = (estimated_seconds * 32.0).ceil() as u32;
            tokens.max(64) // minimum 64 tokens for any audio
        } else {
            BASE_AUDIO_TOKENS
        }
    } else {
        BASE_AUDIO_TOKENS
    }
}

/// Estimate token cost for Gemini inlineData (base64 images/audio in Gemini format).
/// mime_type determines the estimation strategy. data_len is the length of the base64 string.
pub(crate) fn estimate_inline_data_tokens(mime_type: &str, data_len: usize) -> u32 {
    if mime_type.starts_with("image/") {
        // Same logic as estimate_image_tokens_from_url for base64
        let raw_bytes = (data_len * 3) / 4;
        if raw_bytes > 4_000_000 {
            10_000
        } else if raw_bytes > 1_000_000 {
            let factor = raw_bytes as f32 / 1_000_000.0;
            (258.0 * factor * 4.0).ceil() as u32
        } else {
            258
        }
    } else if mime_type.starts_with("audio/") {
        let raw_bytes = (data_len * 3) / 4;
        let estimated_seconds = raw_bytes as f32 / 32_000.0;
        (estimated_seconds * 32.0).ceil().max(64.0) as u32
    } else {
        // Unknown media: treat as text approximation
        estimate_tokens_from_str(&format!("[binary data: {} bytes]", data_len))
    }
}

/// [FIX #3325] Estimate raw input tokens directly from an incoming JSON payload string (OpenAI, Claude, or Gemini format)
/// Used as a fallback when upstream returns an error status (>=400) without token usage metadata.
pub fn estimate_raw_tokens_from_payload(payload: &str) -> u32 {
    if payload.is_empty() {
        return 0;
    }
    if let Ok(json) = serde_json::from_str::<Value>(payload) {
        // Try parsing as OpenAIRequest
        if let Ok(openai_req) = serde_json::from_value::<OpenAIRequest>(json.clone()) {
            return ContextManager::estimate_openai_token_usage(&openai_req);
        }
        // Try parsing as ClaudeRequest
        if let Ok(claude_req) = serde_json::from_value::<ClaudeRequest>(json.clone()) {
            return ContextManager::estimate_token_usage(&claude_req);
        }
        // Try estimating directly from Gemini contents/parts
        return ContextManager::estimate_gemini_token_usage(&json);
    }
    // Fallback: estimate from raw string
    estimate_tokens_from_str(payload)
}

/// Strategy for context purification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurificationStrategy {
    /// Soft purification: Retains recent thinking blocks (~2 turns), removes older ones
    #[allow(dead_code)]
    Soft,
    /// Aggressive purification: Removes ALL thinking blocks to save maximum tokens
    Aggressive,
}

/// Context Manager implementation
pub struct ContextManager;

pub mod claude;
pub mod gemini;
pub mod openai;

#[cfg(test)]
mod tests;
