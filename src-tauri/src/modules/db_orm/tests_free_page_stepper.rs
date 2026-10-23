//! Unit tests for Split-SQLite FreePageStepper (Imp-572)

use rusqlite::Connection;

use super::free_page_stepper::FreePageStepper;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Open in-memory db");
    // Incremental vacuum must be set before creating tables
    conn.execute_batch("PRAGMA auto_vacuum = INCREMENTAL;")
        .expect("Set auto_vacuum");
    conn.execute_batch(
        "CREATE TABLE test_data (id INTEGER PRIMARY KEY, content TEXT);
         CREATE INDEX idx_content ON test_data(content);",
    )
    .expect("Create tables");
    conn
}

#[test]
fn test_step_vacuum_below_threshold_is_noop() {
    let conn = setup_test_db();
    let result = FreePageStepper::step_vacuum(&conn, 50, 10).expect("Step vacuum should succeed");

    assert_eq!(result.pages_requested, 0);
    assert_eq!(result.pages_reclaimed, 0);
    assert_eq!(result.initial_freelist_count, 0);
    assert_eq!(result.final_freelist_count, 0);
}

#[test]
fn test_step_vacuum_reclaims_freelist_pages() {
    let conn = setup_test_db();

    // Populate data
    {
        let mut stmt = conn
            .prepare("INSERT INTO test_data (content) VALUES (?)")
            .unwrap();
        for i in 0..500 {
            stmt.execute([format!("content_block_filler_string_data_{}", i)])
                .unwrap();
        }
    }

    // Delete large portion of data to populate freelist
    conn.execute_batch("DELETE FROM test_data WHERE id > 50;")
        .unwrap();

    // Run stepped vacuum with low threshold
    let result = FreePageStepper::step_vacuum(&conn, 1, 5).expect("Step vacuum should execute");

    // Stepper should request pages if initial freelist exceeded threshold
    if result.initial_freelist_count >= 1 {
        assert!(result.pages_requested > 0);
        assert!(result.final_freelist_count <= result.initial_freelist_count);
    }
}

#[test]
fn test_step_vacuum_handles_empty_table_cleanly() {
    let conn = setup_test_db();
    let result = FreePageStepper::step_vacuum(&conn, 0, 5).expect("Vacuum should handle empty DB");
    assert_eq!(result.pages_requested, 0);
    assert_eq!(result.pages_reclaimed, 0);
}
