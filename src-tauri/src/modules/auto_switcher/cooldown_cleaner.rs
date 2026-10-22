//! Stale Cooldown Memory Cleaner.
//! Periodically cleans expired account switch cooldown timestamps from in-memory caches.

use std::collections::HashMap;

/// Retains only entries whose cooldown expiration timestamp is in the future.
/// Returns the number of expired cooldown records purged.
pub fn clean_expired_cooldowns(
    cooldown_map: &mut HashMap<String, i64>,
    now_sec: i64,
) -> usize {
    let initial_count = cooldown_map.len();
    cooldown_map.retain(|_account_id, expiry_ts| *expiry_ts > now_sec);
    initial_count.saturating_sub(cooldown_map.len())
}
