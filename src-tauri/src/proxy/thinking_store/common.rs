use std::time::Duration;

pub(crate) const MAX_SESSIONS: usize = 2000;
pub(crate) fn max_turns_per_session() -> usize {
    crate::proxy::config::get_thinking_max_memory_turns()
}
pub(crate) const MAX_BYTES_PER_SESSION: usize = 64 * 1024 * 1024;
/// Persist last_accessed at most this often. Fill/hydrate is memory-only between writes.
pub(crate) const TOUCH_PERSIST_INTERVAL: Duration = Duration::from_secs(5 * 60);
pub(crate) fn idle_ttl() -> Duration {
    let days = crate::proxy::config::get_thinking_retention_days().max(1) as u64;
    Duration::from_secs(days.saturating_mul(24 * 60 * 60))
}

// 收集所有的 model 轮次元信息
pub(crate) struct ModelTurnMeta {
    pub(crate) content_idx: usize,
    pub(crate) visible: String,
    pub(crate) norm_visible: String,
    pub(crate) tool_ids: Vec<String>,
    #[allow(dead_code)]
    pub(crate) tool_names: Vec<String>,
    pub(crate) existing_thought: String,
    pub(crate) existing_sig: Option<String>,
    pub(crate) fp: String,
    pub(crate) matched_record_idx: Option<usize>,
    pub(crate) already_complete: bool,
}
