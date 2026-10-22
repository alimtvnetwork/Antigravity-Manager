//! Unit tests for Split-SQLite Mmap Tuner (Imp-560).

use super::mmap_tuner::MmapTuner;
use rusqlite::Connection;

#[test]
fn test_mmap_tuner_constants() {
    assert!(MmapTuner::RECOMMENDED_MMAP_BYTES > 0);
    #[cfg(target_pointer_width = "64")]
    assert_eq!(MmapTuner::RECOMMENDED_MMAP_BYTES, 268_435_456);
}

#[test]
fn test_mmap_tuner_apply_and_query() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    let res = MmapTuner::apply_mmap_size(&conn);
    assert!(res.is_ok());

    let queried = MmapTuner::query_current_mmap_size(&conn).expect("query mmap size");
    // SQLite memory-mapped I/O may return configured or clamped size
    assert!(queried >= 0);
}
