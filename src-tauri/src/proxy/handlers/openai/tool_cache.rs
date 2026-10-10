// In-memory tool-call cache for responses continuations.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde_json::Value;

use super::images_edits::WEBSOCKET_TOOL_CALL_CACHE;

pub fn get_cached_tool_call(call_id: &str) -> Option<Value> {
    if let Some(cache) = WEBSOCKET_TOOL_CALL_CACHE.get() {
        if let Ok(guard) = cache.try_read() {
            return guard.get(call_id).cloned();
        }
    }

    None
}

pub fn insert_cached_tool_call(call_id: String, item: Value) {
    if call_id.is_empty() {
        return;
    }

    let cache = WEBSOCKET_TOOL_CALL_CACHE.get_or_init(|| TokioRwLock::new(HashMap::new()));

    if let Ok(mut guard) = cache.try_write() {
        guard.insert(call_id, item);
    }
}
