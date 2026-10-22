//! Dynamic Page Cache Memory Budget Controller
//!
//! Calculates adaptive SQLite `PRAGMA cache_size` allocations based on system available RAM
//! and active split database count, balancing in-memory caching performance against
//! lightweight background memory footprint.

use tracing::debug;

/// Minimum allowable page cache size in KiB (-2000 KiB / ~2MB).
pub const MIN_CACHE_SIZE_KIB: i32 = 2000;

/// Maximum allowable page cache size in KiB (-64000 KiB / ~64MB).
pub const MAX_CACHE_SIZE_KIB: i32 = 64000;

/// Calculates PRAGMA cache_size argument in KiB (negative integer format per SQLite convention).
pub fn calculate_cache_size_budget(
    available_ram_mb: usize,
    active_db_count: usize,
) -> i32 {
    let db_count = active_db_count.max(1);

    // Dedicate roughly 2% of available RAM divided equally across active databases
    let target_mb_per_db = (available_ram_mb as f64 * 0.02) / (db_count as f64);
    let target_kib = (target_mb_per_db * 1024.0).round() as i32;

    let clamped_kib = target_kib.clamp(MIN_CACHE_SIZE_KIB, MAX_CACHE_SIZE_KIB);

    debug!(
        ram_mb = available_ram_mb,
        db_count = db_count,
        allocated_kib = clamped_kib,
        "Computed SQLite page cache memory budget"
    );

    // Negative value specifies size in KiB in SQLite PRAGMA cache_size
    -clamped_kib
}
