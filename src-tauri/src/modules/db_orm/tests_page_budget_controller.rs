//! Unit tests for Dynamic Page Cache Memory Budget Controller (Imp-564).

use super::page_budget_controller::{
    calculate_cache_size_budget, MAX_CACHE_SIZE_KIB, MIN_CACHE_SIZE_KIB,
};

#[test]
fn test_calculate_cache_size_budget_clamped_min() {
    // Very low available RAM (100MB across 5 DBs): 100 * 0.02 / 5 = 0.4MB = 409 KiB -> clamped to MIN (2000)
    let budget = calculate_cache_size_budget(100, 5);
    assert_eq!(budget, -MIN_CACHE_SIZE_KIB);
    assert_eq!(budget, -2000);
}

#[test]
fn test_calculate_cache_size_budget_clamped_max() {
    // Excessive RAM (128GB across 1 DB): 128000 * 0.02 = 2560MB -> clamped to MAX (64000)
    let budget = calculate_cache_size_budget(128_000, 1);
    assert_eq!(budget, -MAX_CACHE_SIZE_KIB);
    assert_eq!(budget, -64000);
}

#[test]
fn test_calculate_cache_size_budget_normal_allocation() {
    // 16GB RAM across 4 DBs: 16000 * 0.02 / 4 = 80MB = 81920 KiB -> clamped to 64000
    // 8GB RAM across 8 DBs: 8000 * 0.02 / 8 = 20MB = 20480 KiB -> within [2000, 64000]
    let budget = calculate_cache_size_budget(8000, 8);
    assert_eq!(budget, -20480);
}

#[test]
fn test_calculate_cache_size_zero_db_count_handles_safely() {
    let budget = calculate_cache_size_budget(8000, 0);
    // Active db count of 0 is treated as 1
    assert!(budget < 0);
}
