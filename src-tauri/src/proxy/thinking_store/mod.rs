//! Server-side full thinking-block store.
//!
//! Captures untruncated thought text + thoughtSignature from upstream Gemini
//! responses, then precisely re-injects them into the next request's `contents`
//! even when the client dropped / truncated thinking.
//!
//! Matching is content-based (visible assistant text + tool ids), not turn
//! index, so OpenAI / Anthropic / Gemini packet shapes can differ.
//!
//! Isolation key = `{tenant}:{client_session_id}`:
//! - tenant is a hash of the caller's API key
//! - client_session_id prefers `X-Session-Id` / body `session_id`

mod common;
mod gemini_capture;
mod hashing;
mod model;
mod session_extract;
mod session_scope;
mod signatures;
mod store;
mod store_restore;
mod store_restore_apply;
mod store_session;
mod thought_parts;
mod turn_accumulator;

#[cfg(test)]
mod tests_common;
#[cfg(test)]
mod tests_finalize;
#[cfg(test)]
mod tests_finalize_extra;
#[cfg(test)]
mod tests_ingest;
#[cfg(test)]
mod tests_record_restore;
#[cfg(test)]
mod tests_restore_phases;
#[cfg(test)]
mod tests_sessions;
#[cfg(test)]
mod tests_signature_placement;

pub use gemini_capture::{
    capture_gemini_contents, capture_gemini_parts, capture_gemini_parts_with_anchor,
    capture_gemini_response, capture_gemini_response_with_preceding,
    finalize_gemini_contents_thinking, finalize_gemini_contents_thinking_with_model,
    hydrate_gemini_contents, hydrate_gemini_contents_with_model,
};
pub use hashing::{canonical_json_hash, compute_causal_anchor, fingerprint, synthesize_tool_id};
pub use model::{
    any_model_forces_server_thinking, model_forces_server_thinking, ThinkingRecord,
    SENTINEL_SIGNATURE,
};
pub use session_extract::{
    collect_session_semantic_headers, derive_blended_session_id, explicit_session_id,
    explicit_session_id_with_query, extract_body_session_id, extract_query_session_id,
    extract_session_from_query_str, is_session_semantic_header, sanitize_session_id,
};
pub use session_scope::{EndSessionResult, SessionScope};
pub use signatures::{
    ensure_google_claude_thought_signature, ensure_raw_claude_thought_signature,
    is_claude_signature, is_likely_gemini_signature, is_real_signature,
    normalize_signature_for_comparison, signatures_match,
};
pub use store::ThinkingStore;
pub use thought_parts::{
    extract_think_tags, is_meaningful_thought, is_placeholder_thought, is_thought_part,
    place_turn_signature,
};
pub use turn_accumulator::TurnAccumulator;

pub(crate) use common::{
    idle_ttl, max_turns_per_session, ModelTurnMeta, MAX_BYTES_PER_SESSION, MAX_SESSIONS,
    TOUCH_PERSIST_INTERVAL,
};
#[cfg(test)]
pub(crate) use gemini_capture::contents_have_capturable_thought;
pub(crate) use gemini_capture::is_model_or_assistant;
pub(crate) use hashing::{inspect_parts_with_anchor, record_bytes};
pub(crate) use session_extract::{client_id_from_store_key, tenant_from_headers};
pub(crate) use thought_parts::{
    is_capturable_thought, normalize_ws, part_has_signature, turn_needs_restore,
};
