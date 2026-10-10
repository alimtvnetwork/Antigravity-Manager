//! Repo DB: state

use super::models::ActivePrompt;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::Once;
use std::sync::OnceLock;

static MEMORY_ACTIVE_PROMPTS: OnceLock<Mutex<HashMap<String, ActivePrompt>>> = OnceLock::new();
static DISPATCHED_PROMPTS_CACHE: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static ACTIVE_AGY_WORKERS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
static STARTUP_PURGE_ONCE: Once = Once::new();

pub(crate) fn get_memory_prompts_map() -> &'static Mutex<HashMap<String, ActivePrompt>> {
    MEMORY_ACTIVE_PROMPTS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn get_active_agy_workers() -> &'static Mutex<HashMap<String, u32>> {
    ACTIVE_AGY_WORKERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_dispatched_prompts_cache() -> &'static Mutex<HashSet<String>> {
    DISPATCHED_PROMPTS_CACHE.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn reset_dispatched_prompts_cache() {
    if let Ok(mut set) = get_dispatched_prompts_cache().lock() {
        set.clear();
    }
}

/// Retrieve an active prompt from in-memory cache if available
pub fn get_memory_prompt(prompt_id: &str) -> Option<ActivePrompt> {
    if let Ok(map) = get_memory_prompts_map().lock() {
        return map.get(prompt_id).cloned();
    }
    None
}
