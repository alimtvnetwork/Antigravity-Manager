#[cfg(test)]
mod tests {
    use super::super::schema_integrity_gate::SchemaIntegrityGate;
    use rusqlite::Connection;

    #[test]
    fn test_schema_integrity_all_tables_present() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE prompts (id INTEGER PRIMARY KEY, prompt TEXT);",
            [],
        )
        .unwrap();

        let report = SchemaIntegrityGate::verify_tables(&conn, &["users", "prompts"]).unwrap();
        assert!(report.is_valid);
        assert!(report.missing_tables.is_empty());
        assert_eq!(report.existing_tables.len(), 2);
    }

    #[test]
    fn test_schema_integrity_missing_table_detected() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY);", [])
            .unwrap();

        let report =
            SchemaIntegrityGate::verify_tables(&conn, &["users", "accounts", "prompts"]).unwrap();
        assert!(!report.is_valid);
        assert_eq!(report.missing_tables, vec!["accounts", "prompts"]);
    }

    #[test]
    fn test_schema_integrity_case_insensitive_matching() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE ActivePrompts (id INTEGER PRIMARY KEY);", [])
            .unwrap();

        let report = SchemaIntegrityGate::verify_tables(&conn, &["activeprompts"]).unwrap();
        assert!(report.is_valid);
        assert!(report.missing_tables.is_empty());
    }

    #[test]
    fn test_schema_integrity_system_tables_excluded() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE items (id INTEGER PRIMARY KEY AUTOINCREMENT);",
            [],
        )
        .unwrap();

        // sqlite_sequence is created automatically for AUTOINCREMENT
        let report = SchemaIntegrityGate::verify_tables(&conn, &["items"]).unwrap();
        assert!(report.is_valid);
        assert!(!report
            .existing_tables
            .contains(&"sqlite_sequence".to_string()));
    }

    #[test]
    fn test_schema_integrity_empty_expected_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let report = SchemaIntegrityGate::verify_tables(&conn, &[]).unwrap();
        assert!(report.is_valid);
        assert!(report.missing_tables.is_empty());
    }
}
