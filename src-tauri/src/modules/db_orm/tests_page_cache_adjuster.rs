//! Unit tests for SQLite Page Cache Adjuster (Imp-556).

use super::page_cache_adjuster::PageCacheAdjuster;
use rusqlite::Connection;

#[test]
fn test_set_cache_size_within_valid_bounds() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    // Set 4096 KiB (4MB)
    let res = PageCacheAdjuster::set_cache_size_kib(&conn, 4096);
    assert!(res.is_ok());

    let val = PageCacheAdjuster::get_current_cache_size(&conn).expect("get cache size");
    // SQLite returns negative value for KiB cache size
    assert_eq!(val, -4096);
}

#[test]
fn test_set_cache_size_clamping_lower_bound() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    // Setting 100 KiB clamps up to 1024 KiB minimum
    let res = PageCacheAdjuster::set_cache_size_kib(&conn, 100);
    assert!(res.is_ok());

    let val = PageCacheAdjuster::get_current_cache_size(&conn).expect("get cache size");
    assert_eq!(val, -1024);
}

#[test]
fn test_set_cache_size_clamping_upper_bound() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    // Setting 100_000 KiB clamps down to 65536 KiB maximum (64MB)
    let res = PageCacheAdjuster::set_cache_size_kib(&conn, 100_000);
    assert!(res.is_ok());

    let val = PageCacheAdjuster::get_current_cache_size(&conn).expect("get cache size");
    assert_eq!(val, -65536);
}
